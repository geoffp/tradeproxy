use config::{Config, ConfigError, Environment, File, FileFormat};
use directories_next::BaseDirs;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{
    collections::HashSet,
    path::Path,
    result::Result,
    sync::{RwLock, RwLockReadGuard},
};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub listen_port: u16,
    pub log_dir: String,
    pub long_bot_id: u64,
    pub short_bot_id: u64,
    pub email_token: String,
    pub tradingview_api_ips: HashSet<String>,
    pub log_path: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        let base_dirs = BaseDirs::new();

        // On Linux, this will be ~/.config
        let user_config_dir: &Path = match &base_dirs {
            Some(base_dirs) => base_dirs.config_dir(),
            None => panic!("Can't find user config directory!"),
        };

        // Start off by merging in the "default" configuration file
        // s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        // let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        // s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        s.merge(File::with_name("config/default.yaml").required(false))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        // AND it should only be loaded when we're not running unit tests -- in that case, we should use the defaults

        let mut log_dir_str: Option<String> = None;

        let tp_config_dir = if cfg!(test) {
            None
        } else if cfg!(debug_assertions) {
            Some(user_config_dir.join("tradeproxy-dev"))
        } else {
            Some(user_config_dir.join("tradeproxy"))
        };

        eprintln!("Config dir: {:?}", tp_config_dir);

        if let Some(dir) = &tp_config_dir {
            log_dir_str = dir.join("log").to_str().map(String::from);
            let config_path = dir.join("config.yaml");
            s.merge(
                File::from(config_path)
                    .format(FileFormat::Yaml)
                    .required(true),
            )?;
        };

        s.set("log_path", log_dir_str).unwrap();

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("tp"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = match Settings::new() {
        Ok(s) => RwLock::new(s),
        Err(e) => panic!("Error loading config: {:?}", e),
    };
}

pub fn get_settings() -> RwLockReadGuard<'static, Settings> {
    SETTINGS.read().unwrap()
}
