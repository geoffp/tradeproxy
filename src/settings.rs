use std::{result::Result, collections::HashSet, sync::{RwLock, RwLockReadGuard}, path::Path};
use serde::Deserialize;
use config::{ConfigError, Config, File, FileFormat, Environment};
use directories_next::BaseDirs;
use lazy_static::lazy_static;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub listen_port: u16,
    pub log_dir: String,
    pub long_bot_id: u64,
    pub short_bot_id: u64,
    pub email_token: String,
    pub tradingview_api_ips: HashSet<String>
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        let base_dirs = BaseDirs::new();

        // On Linux, this will be ~/.config
        let user_config_dir: &Path = match &base_dirs {
            Some(base_dirs) => base_dirs.config_dir(),
            None => panic!("Can't find user config directory!")
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
        if !cfg!(test) {
            s.merge(File::from(user_config_dir.join("tradeproxy").join("config.yaml")).format(FileFormat::Yaml).required(true))?;
        }

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
        Err(e) => panic!("Error loading config: {:?}", e)
    };
}

pub fn get_settings() -> RwLockReadGuard<'static, Settings> {
    SETTINGS.read().unwrap()
}
