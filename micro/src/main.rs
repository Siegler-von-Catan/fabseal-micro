// #[macro_use]
// extern crate lazy_static;

use actix_redis::RedisSession;
use actix_web::cookie::SameSite;
use actix_web::web::Data;
use redis::IntoConnectionInfo;
use time::Duration;

use rand::Rng;

use lapin::{ConnectionProperties};

// use actix_storage_sled::{actor::ToActorExt, SledConfig};
use actix_storage::Storage;
use actix_storage_redis::{RedisBackend};

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};

use log::{debug, info};

mod site;
use site::create::create_service;
use site::rmq_ops::rmq_declare;

mod settings;

use settings::Settings;


async fn create_rmq_pool(
    db_url: &str,
) -> bb8::Pool<bb8_lapin::LapinConnectionManager> {
    // Create RabbitMQ pool
    let manager = bb8_lapin::LapinConnectionManager::new(
        db_url,
        ConnectionProperties::default()
        // .with_tokio()
    );
    bb8::Pool::builder()
        .max_size(15)
        .build(manager)
        .await
        .unwrap()
}

const COOKIE_DURATION: Duration = Duration::hour();

fn create_redis_session(
    settings: Settings,
    key: &[u8]
) -> RedisSession {
    RedisSession::new(settings.redis.address.clone(), key)
        // .domain("localhost")
        .cookie_name("fabseal_session")
        .cookie_path("/api/v1")
        .cookie_secure(!settings.debug)
        // .expires_in_time(COOKIE_DURATION)
        .cookie_http_only(true)
        .cookie_max_age(COOKIE_DURATION)
        .cookie_same_site(SameSite::Strict)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let settings = Settings::new().unwrap();
    debug!("settings: {:?}", settings);

    let rmq_pool = create_rmq_pool(settings.amqp.address.as_str()).await;

    let rr = rmq_declare(rmq_pool.clone()).await.unwrap();
    info!("{:?}", rr);

    // let ci = redis::ConnectionInfo { addr: Box::new(settings.redis.address), db: 0, passwd: settings.redis.password };
    let ci = format!("redis://{}", settings.redis.address.as_str()).into_connection_info().unwrap();
    let store = RedisBackend::connect(ci).await.unwrap();
    let storage = Storage::build().expiry_store(store).finish();

    let key: [u8; 32] = rand::thread_rng().gen();

    let ep = settings.http_endpoint.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(storage.clone())
            .app_data(Data::new(rmq_pool.clone()))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(Logger::default())
            .wrap(create_redis_session(settings.clone(), &key))
            .service(web::scope("/api/v1").configure(create_service))
    })
    .bind(ep)?
    .run()
    .await
}
