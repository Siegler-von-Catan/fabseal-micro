use config::{ConfigError, Config, File, Environment};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub address: String,
    pub db_id: Option<i64>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub debug: bool,
    pub dmstl_directory: String,
    pub redis: RedisSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("fabseal"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}