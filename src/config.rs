use std::fs;
use mlua::{Lua, Result};

#[derive(Debug)]
pub struct Config {
    pub direct_connection: bool,
    pub target_host: String,
    pub username: String,
    pub vps_host: Option<String>,
    pub vps_username: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = format!("{}/.config/remo/config.lua", std::env::var("HOME").unwrap());
        let config_code = fs::read_to_string(&config_path)
            .expect("Failed to load config.lua");

        let lua = Lua::new();
        lua.load(&config_code).exec()?;

        let globals = lua.globals();
        let direct_connection: bool = globals.get("direct_connection")?;

        if direct_connection {
            Ok(Config {
                direct_connection,
                target_host: globals.get("target_host")?,
                username: globals.get("username")?,
                vps_host: None,
                vps_username: None,
            })
        } else {
            Ok(Config {
                direct_connection,
                target_host: globals.get("homelab_host")?,
                username: globals.get("homelab_username")?,
                vps_host: Some(globals.get("vps_host")?),
                vps_username: Some(globals.get("vps_username")?),
            })
        }
    }
}

