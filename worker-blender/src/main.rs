use log::debug;

use color_eyre::eyre::Result;

mod settings;
use settings::Settings;
mod worker;
use crate::worker::Worker;

fn main() -> Result<()> {
    color_eyre::install()?;

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let settings = Settings::new()?;
    debug!("settings: {:?}", settings);

    let mut worker = Worker::create(settings)?;
    worker.run();

    Ok(())
}
