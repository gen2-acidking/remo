use std::process::Command;
use std::io::{self, Write};
use log::{info, error};

pub fn run_setup() {
    info!("🚀 Welcome to Remote Governor Setup");

    let direct_connection = prompt_bool("Are you setting up a direct connection (y/n)? ");
    
    let username = prompt("Enter the server username: ");
    let ip = prompt("Enter the server IP address: ");

    let vps_username;
    let vps_ip;

    if !direct_connection {
        vps_username = Some(prompt("Enter the VPS username: "));
        vps_ip = Some(prompt("Enter the VPS IP address: "));
    } else {
        vps_username = None;
        vps_ip = None;
    }

    let key_path = format!("{}/.ssh/id_{}", std::env::var("HOME").unwrap(), if direct_connection { "server" } else { "vps" });

    info!("🔑 Checking for existing SSH keys...");

    if !std::path::Path::new(&key_path).exists() {
        info!("➡️ No key found, generating a new one...");
        generate_key(&key_path);
    } else {
        info!("✅ Key found at {} — reusing it.", key_path);
    }

    // Add key to ssh-agent
    info!("➡️ Adding key to SSH agent...");
    add_key_to_agent(&key_path);
    
    // Copy key to the target server
    if direct_connection {
        copy_key(&key_path, &username, &ip);
    } else {
        if let (Some(vps_username_ref), Some(vps_ip_ref)) = (&vps_username, &vps_ip) {
            copy_key_via_vps(&key_path, &username, &ip, vps_username_ref, vps_ip_ref);
        } else {
            error!("VPS information missing.");
            return;
        }
    }

    // Write to ~/.ssh/config
    write_ssh_config(&key_path, &username, &ip, &vps_username, &vps_ip);

    info!("✅ Setup complete! Try `ssh {}` to test the connection.", username);
}


fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");

    input.trim().to_string()
}

fn prompt_bool(msg: &str) -> bool {
    loop {
        let input = prompt(msg);
        match input.to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => error!("Invalid input, please enter y/n."),
        }
    }
}

fn generate_key(key_path: &str) {
    let output = Command::new("ssh-keygen")
        .arg("-t").arg("ed25519")
        .arg("-f").arg(key_path)
        .arg("-N").arg("")
        .output()
        .expect("Failed to generate SSH key");

    if output.status.success() {
        info!("✅ Key generated at {}", key_path);
    } else {
        error!("❌ Failed to generate key: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn add_key_to_agent(key_path: &str) {
    let output = Command::new("ssh-add")
        .arg(key_path)
        .output()
        .expect("Failed to add key to agent");

    if output.status.success() {
        info!("✅ Key added to SSH agent.");
    } else {
        error!("❌ Failed to add key: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn copy_key(key_path: &str, username: &str, ip: &str) {
    info!("➡️ Copying key to target server...");
    let output = Command::new("ssh-copy-id")
        .arg("-i").arg(format!("{}.pub", key_path))
        .arg(format!("{}@{}", username, ip))
        .output()
        .expect("Failed to copy key");

    if output.status.success() {
        info!("✅ Key copied to target server.");
    } else {
        error!("❌ Failed to copy key: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn copy_key_via_vps(key_path: &str, username: &str, ip: &str, vps_username: &str, vps_ip: &str) {
    info!("➡️ Copying key through VPS...");

    let intermediate_copy = format!(
        "scp {}.pub {}@{}:~/.ssh/temp_key.pub",
        key_path,
        vps_username,
        vps_ip
    );

    let final_copy = format!(
        "ssh {}@{} 'cat ~/.ssh/temp_key.pub >> ~/.ssh/authorized_keys && rm ~/.ssh/temp_key.pub'",
        username,
        ip
    );

    if Command::new("sh").arg("-c").arg(&intermediate_copy).status().unwrap().success() &&
       Command::new("sh").arg("-c").arg(&final_copy).status().unwrap().success() {
        info!("✅ Key copied to homelab through VPS.");
    } else {
        error!("❌ Failed to copy key to homelab.");
    }
}

fn write_ssh_config(key_path: &str, username: &str, ip: &str, vps_username: &Option<String>, vps_ip: &Option<String>) {
    let config_entry = if let (Some(vps_username), Some(vps_ip)) = (vps_username, vps_ip) {
        format!(
            "Host {}\n    HostName {}\n    User {}\n    IdentityFile {}\n    ProxyJump {}@{}\n\n",
            username, ip, username, key_path, vps_username, vps_ip
        )
    } else {
        format!(
            "Host {}\n    HostName {}\n    User {}\n    IdentityFile {}\n\n",
            username, ip, username, key_path
        )
    };

    let config_path = format!("{}/.ssh/config", std::env::var("HOME").unwrap());
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .expect("Failed to open ssh config")
        .write_all(config_entry.as_bytes())
        .expect("Failed to write to ssh config");

    info!("✅ SSH config updated.");
}

