use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};

use fabseal_micro_common::settings::{Limits, RedisSettings};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub dmstl_directory: PathBuf,

    #[serde(default)]
    pub redis: RedisSettings,

    #[serde(default)]
    pub limits: Limits,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();

        // Include config.toml entries
        s.merge(File::with_name("config"))?;

        // Add in settings from the environment (with a prefix of FABSEAL)
        // Eg.. `FABSEAL_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("fabseal"))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}
