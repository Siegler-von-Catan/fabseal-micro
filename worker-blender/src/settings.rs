use config::{ConfigError, Config, File, Environment};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AmqpSettings {
    pub address: String,
}

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
    pub http_endpoint: String,
    pub amqp: AmqpSettings,
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

        // Now that we're done, let's access our configuration
        println!("debug: {:?}", s.get_bool("debug"));
        println!("database: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}