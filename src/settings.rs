use std::{result::Result, env, collections::HashSet, sync::{RwLock, RwLockReadGuard}};
use serde::Deserialize;
use config::{ConfigError, Config, File, FileFormat, Environment};
use lazy_static::lazy_static;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub long_bot_id: u64,
    pub short_bot_id: u64,
    pub email_token: String,
    pub tradingview_api_ips: HashSet<String>
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();

        // Start off by merging in the "default" configuration file
        // s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("/home/geoff/.tradeproxy.yaml").format(FileFormat::Yaml).required(true))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("app"))?;

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
