#[macro_use]
extern crate lazy_static;

use lapin::{Connection, ConnectionProperties, Result as LAResult};

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};

use log::{info, debug};

mod site;
use site::create::{create_service, rmq_declare};
use tokio_amqp::LapinTokioExt;

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
        ConnectionProperties::default().with_tokio()
    );
    bb8::Pool::builder()
        .max_size(15)
        .build(manager)
        .await
        .unwrap()
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let rmq_db_url = std::env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let rmq_pool = create_rmq_pool(rmq_db_url.as_str()).await;

    // let pg_db_url = "host=localhost user=testuser";
    // let pg_pool = create_pg_pool(
    //     pg_db_url,
    // ).await;

    let http_endpoint = std::env::var("HTTP_ADDR")
    .unwrap_or_else(|_| "127.0.0.1:8080".into());

    let rr = rmq_declare(rmq_pool.clone()).await.unwrap();
    info!("{:?}", rr);

    HttpServer::new(move || {
        App::new()
            .data(rmq_pool.clone())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(Logger::default())
            .service(web::scope("/api/v1").configure(create_service))
    })
    .bind(http_endpoint)?
    .run()
    .await
}
