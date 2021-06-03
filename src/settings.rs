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
#[serde(default)]
pub struct Settings {
    pub listen_port: u16,
    pub long_bot_id: u64,
    pub short_bot_id: u64,
    pub email_token: String,
    pub tradingview_api_ips: HashSet<String>,
    pub log_path: String,
    pub request_server: String,
    pub request_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            email_token: "89abcdef-789a-bcde-f012-456789abcdef".into(),
            listen_port: 3137,
            log_path: ".".into(),
            long_bot_id: 1234567,
            request_server: "https://3commas.io".into(),
            request_path: "/trade_signal/trading_view".into(),
            short_bot_id: 7654321,
            tradingview_api_ips: [
                "52.89.214.238",
                "34.212.75.30",
                "54.218.53.128",
                "52.32.178.7",
            ].iter().cloned().map(String::from).collect(),
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        let base_dirs = BaseDirs::new();

        // On Linux, this will be ~/.config
        let user_config_dir = get_user_config_dir(&base_dirs);

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
        let tp_config_dir = if cfg!(test) {
            None
        } else if cfg!(debug_assertions) {
            Some(user_config_dir.join("tradeproxy-dev"))
        } else {
            Some(user_config_dir.join("tradeproxy"))
        };

        eprintln!("Config dir: {:?}", tp_config_dir);

        // If we've found a user config directory:
        // - Set up the log path
        // - Find the config file's path
        // - Merge that file into self
        let log_dir: String = if let Some(config_dir) = &tp_config_dir {
            // Load up the config file
            let config_file_path = config_dir.join("config.yaml");
            s.merge(
                File::from(config_file_path)
                    .format(FileFormat::Yaml)
                    .required(true),
            )?;

            // Return the log directory
            config_dir.join("log").to_str().unwrap().into()
        } else {
            // If there's not user config directory, we must log...here
            ".".into()
        };

        s.set("log_path", log_dir).unwrap();

        // Add in settings from the environment (with a prefix of TP)
        // Eg.. `TP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("tp"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

fn get_user_config_dir<'a>(base_dirs: &'a Option<BaseDirs>) -> &'a Path {
    let user_config_dir: &'a Path = match &base_dirs {
        Some(base_dirs) => base_dirs.config_dir(),
        None => panic!("Can't find user config directory!"),
    };
    user_config_dir
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
