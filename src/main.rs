mod setup;
mod config;

use clap::{Arg, Command};
use log::{info, error};
use env_logger;
use std::process::Command as ShellCommand;

fn connect() {
    let config = match config::Config::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("Failed to load configuration: {}", err);
            return;
        }
    };

    if config.direct_connection {
        info!("Using direct SSH connection to {}...", config.target_host);
        let status = ShellCommand::new("ssh")
            .arg(format!("{}@{}", config.username, config.target_host))
            .status();

        if let Ok(status) = status {
            if status.success() {
                info!("Connected to homelab.");
            } else {
                error!("Failed to connect to homelab.");
            }
        } else {
            error!("SSH process failed.");
        }
    } else {
        info!("Using two-step connection via VPS ({})...", config.vps_host.as_ref().unwrap());
        let vps_ssh_command = format!(
            "ssh {}@{} -t 'ssh {}@{}'",
            config.vps_username.as_ref().unwrap(),
            config.vps_host.as_ref().unwrap(),
            config.username,
            config.target_host
        );

        let status = ShellCommand::new("sh")
            .arg("-c")
            .arg(&vps_ssh_command)
            .status();

        if let Ok(status) = status {
            if status.success() {
                info!("Connected to homelab via VPS.");
            } else {
                error!("Failed to connect to homelab via VPS.");
            }
        } else {
            error!("SSH process failed.");
        }
    }
}

fn scp_put(filepath: &str) {
    let config = match config::Config::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("Failed to load configuration: {}", err);
            return;
        }
    };

    if config.direct_connection {
        info!("SCP transferring `{}` directly to homelab...", filepath);
        let status = ShellCommand::new("scp")
            .arg(filepath)
            .arg(format!("{}@{}:~/", config.username, config.target_host))
            .status();

        if let Ok(status) = status {
            if status.success() {
                info!("File `{}` successfully copied to homelab.", filepath);
            } else {
                error!("SCP to homelab failed.");
            }
        } else {
            error!("SCP process failed.");
        }
    } else {
        info!("SCP transferring `{}` via VPS ({})...", filepath, config.vps_host.as_ref().unwrap());

        // Step 1: Copy to VPS
        let intermediate_copy = format!(
            "scp {} {}@{}:~/",
            filepath,
            config.vps_username.as_ref().unwrap(),
            config.vps_host.as_ref().unwrap()
        );

        let status1 = ShellCommand::new("sh")
            .arg("-c")
            .arg(&intermediate_copy)
            .status();

        if let Ok(status) = status1 {
            if status.success() {
                info!("File `{}` successfully copied to VPS.", filepath);
            } else {
                error!("Intermediate SCP to VPS failed.");
                return;
            }
        } else {
            error!("Intermediate SCP process failed.");
            return;
        }

        // Step 2: Relay to Homelab
        let final_copy = format!(
            "ssh {}@{} 'scp ~/{} {}@{}:~/'",
            config.vps_username.as_ref().unwrap(),
            config.vps_host.as_ref().unwrap(),
            filepath,
            config.username,
            config.target_host
        );

        let status2 = ShellCommand::new("sh")
            .arg("-c")
            .arg(&final_copy)
            .status();

        if let Ok(status) = status2 {
            if status.success() {
                info!("File `{}` successfully relayed to homelab.", filepath);
            } else {
                error!("Final SCP to homelab failed.");
            }
        } else {
            error!("Final SCP process failed.");
        }
    }
}

fn main() {
    env_logger::init(); // Initialize logging

    let matches = Command::new("Remote Governor")
        .version("0.1")
        .author("Your Name")
        .about("Remote access, SCP, and compilation utility")
        .subcommand(
            Command::new("connect")
                .about("Connects to the homelab (direct or via VPS)")
        )
        .subcommand(
            Command::new("put")
                .about("SCP file to homelab")
                .arg(Arg::new("file")
                    .required(true)
                    .help("File to copy to homelab"))
        )
        .subcommand(
            Command::new("setup")
                .about("Set up key-based SSH authentication")
        )
        .get_matches();

    match matches.subcommand() {
        Some(("connect", _)) => connect(),
        Some(("put", sub_m)) => {
            let filepath = sub_m.get_one::<String>("file").unwrap();
            scp_put(filepath);
        }
        Some(("setup", _)) => {
            setup::run_setup();
        }
        _ => {
            eprintln!("Unknown command. Use --help for usage.");
        }
    }
}

