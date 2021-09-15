use actix_redis::RedisSession;
use actix_session::CookieSession;
use actix_web::{cookie::SameSite, http, web::Data};

use actix_cors::Cors;

use rand::Rng;

use actix_web::{middleware::Logger, web, App, HttpServer};

use actix_redis::RedisActor;

use log::debug;

mod site;
use site::create::create_service;

mod settings;

use settings::Settings;
use time::Duration;

mod prepare_image;

const COOKIE_DURATION: Duration = Duration::hour();

fn create_cookie_session(settings: &Settings, key: &[u8]) -> CookieSession {
    let s = CookieSession::private(key)
        .max_age(COOKIE_DURATION.whole_seconds())
        .name("fabseal_session")
        .path("/api/v1/create")
        .http_only(true)
        .same_site(SameSite::Strict);

    if settings.debug {
        s.secure(false)
    } else {
        match &settings.http.cookie_domain {
            Some(d) => s.secure(true).domain(d),
            _ => s.secure(true),
        }
    }
}

fn create_redis_session(settings: &Settings, key: &[u8]) -> RedisSession {
    let s = RedisSession::new(settings.redis.address.clone(), key)
        .ttl(settings.limits.session_ttl)
        .cookie_name("fabseal_session")
        .cookie_path("/api/v1/create")
        .cookie_http_only(true)
        .cookie_max_age(COOKIE_DURATION)
        .cookie_same_site(SameSite::Strict);

    if settings.debug {
        s.cookie_secure(false)
    } else {
        match &settings.http.cookie_domain {
            Some(d) => s.cookie_secure(true).cookie_domain(d),
            _ => s.cookie_secure(true),
        }
    }
}

fn build_cors(settings: &Settings) -> actix_cors::Cors {
    if settings.debug {
        Cors::permissive()
    } else {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::ACCEPT])
            .max_age(3600)
            .supports_credentials();
        for origin in &settings.http.cors_origins {
            cors = cors.allowed_origin(origin);
        }
        cors
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let settings = Settings::new().unwrap();
    debug!("settings: {:?}", settings);

    let key: [u8; 32] = rand::thread_rng().gen();

    let ep = settings.http.endpoint.clone();
    HttpServer::new(move || {
        let redis = RedisActor::start(settings.redis.address.as_str());
        let cors = build_cors(&settings);

        App::new()
            .app_data(Data::new(redis))
            .app_data(Data::new(settings.clone()))
            // .wrap(actix_web::middleware::Compress::default())
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(create_cookie_session(&settings, &key))
            .service(web::scope("/api/v1").configure(create_service))
    })
    .bind(ep)?
    .run()
    .await
}
