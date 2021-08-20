// #[macro_use]
// extern crate lazy_static;

use actix_redis::RedisSession;
use actix_web::cookie::SameSite;
use actix_web::web::Data;
use time::Duration;

use rand::Rng;

use lapin::{ConnectionProperties};

// use actix_storage_sled::{actor::ToActorExt, SledConfig};
use actix_storage::Storage;
use actix_storage_redis::{RedisBackend, ConnectionInfo, ConnectionAddr};

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_session::CookieSession;

use log::info;

mod site;
use site::create::{create_service, rmq_declare};
// use tokio_amqp::LapinTokioExt;

/*
async fn create_pg_pool(
    db_url: &str,
) -> bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>> {
    let pg_mgr =
        bb8_postgres::PostgresConnectionManager::new_from_stringlike(db_url, tokio_postgres::NoTls)
            .unwrap();

    bb8::Pool::builder()
        .max_size(15)
        .build(pg_mgr)
        .await
        .unwrap()
}
 */

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

const THREADS_NUMBER: usize = 4;

fn create_redis_session(
    redis_endpoint: &str,
    key: &[u8],
    dev_env: bool
) -> RedisSession {
    RedisSession::new(redis_endpoint, key)
        // .domain("localhost")
        .cookie_name("fabseal_session")
        .cookie_path("/api/v1")
        .cookie_secure(!dev_env)
        // .expires_in_time(COOKIE_DURATION)
        .cookie_http_only(true)
        .cookie_max_age(COOKIE_DURATION)
        .cookie_same_site(SameSite::Strict)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let dev_env: bool = std::env::var("DEV").is_ok();

    let rmq_db_url = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let rmq_pool = create_rmq_pool(rmq_db_url.as_str()).await;

    // let pg_db_url = "host=localhost user=testuser";
    // let pg_pool = create_pg_pool(
    //     pg_db_url,
    // ).await;

    let http_endpoint = std::env::var("HTTP_ADDR")
    .unwrap_or_else(|_| "127.0.0.1:8080".into());

    let redis_endpoint = std::env::var("REDIS_ADDR")
    .unwrap_or_else(|_| "127.0.0.1:6379".into());

    let rr = rmq_declare(rmq_pool.clone()).await.unwrap();
    info!("{:?}", rr);

    /*
    // Refer to sled's documentation for more options
    let sled_db = SledConfig::default().temporary(true);

    // Open the database and make an actor(not started yet)
    let actor = sled_db.to_actor()?;

    let store = actor
                // If you want to scan the database on start for expiration
                .scan_db_on_start(true)
                // If you want the expiration thread to perform deletion instead of soft deleting items
                .perform_deletion(true)
                // Finally start the actor
                .start(THREADS_NUMBER);

    let storage = Storage::build().expiry_store(store).finish();
    */


    let store = RedisBackend::connect_default().await.unwrap();

    let storage = Storage::build().expiry_store(store).finish();

    let key: [u8; 32] = rand::thread_rng().gen();

    HttpServer::new(move || {
        App::new()
            .app_data(storage.clone())
            .app_data(Data::new(rmq_pool.clone()))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(Logger::default())
            .wrap(create_redis_session(&redis_endpoint, &key, dev_env))
            .service(web::scope("/api/v1").configure(create_service))
    })
    .bind(http_endpoint)?
    .run()
    .await
}
