use actix_redis::RedisSession;
use actix_web::{cookie::SameSite, web::Data};

use fabseal_micro_common::SESSION_TTL_SECONDS;
use rand::Rng;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};

use actix_redis::RedisActor;

use log::debug;

mod site;
use site::create::create_service;

mod settings;

use settings::Settings;
use time::Duration;

const COOKIE_DURATION: Duration = Duration::hour();

fn create_redis_session(
    settings: Settings,
    key: &[u8]
) -> RedisSession {
    let s = RedisSession::new(settings.redis.address.clone(), key)
        .ttl(SESSION_TTL_SECONDS)
        .cookie_name("fabseal_session")
        .cookie_path("/api/v1/create")
        .cookie_http_only(true)
        .cookie_max_age(COOKIE_DURATION)
        .cookie_same_site(SameSite::Strict);

    if settings.debug {
        s.cookie_secure(false)
    } else {
        match settings.domain {
            Some(d) => {
                s
                    .cookie_secure(true)
                    .cookie_domain(&d)
            },
            _ => {
                s
                    .cookie_secure(true)
            },
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let settings = Settings::new().unwrap();
    debug!("settings: {:?}", settings);

    let key: [u8; 32] = rand::thread_rng().gen();

    let ep = settings.http_endpoint.clone();
    HttpServer::new(move || {
        let redis = RedisActor::start(settings.redis.address.as_str());

        App::new()
            .app_data(Data::new(redis))
            // .wrap(actix_web::middleware::Compress::default())
            .wrap(Logger::default())
            .wrap(create_redis_session(settings.clone(), &key))
            .service(web::scope("/api/v1").configure(create_service))
    })
    .bind(ep)?
    .run()
    .await
}
