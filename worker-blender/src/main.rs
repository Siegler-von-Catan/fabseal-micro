use log::debug;

mod settings;
use settings::Settings;
mod worker;
use crate::worker::Worker;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let settings = Settings::new().unwrap();
    debug!("settings: {:?}", settings);

    let mut worker = Worker::create().expect("worker creation");
    worker.run();
}