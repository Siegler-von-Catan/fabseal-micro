use actix::Addr;
use actix_web::{HttpResponse, Result as AWResult, get, post, web};
use actix_session::Session;
use actix_multipart as mp;
use actix_redis::{Command, RedisActor, RespValue};
// use redis_async::resp_array;

use futures_util::TryStreamExt;

use rand::Rng;

use log::{debug, error, info};

use fabseal_micro_common::{FABSEAL_SUBMISSION_QUEUE, FABSEAL_SUBMISSION_QUEUE_LIMIT, IMAGE_EXPIRATION_SECONDS, image_key, result_key};
use redis_async::resp_array;

use crate::site::types::*;
use crate::site::util::*;

#[get("/public/result")]
async fn fetch_model(
    info: web::Query<ResultRequestInfo>
) -> AWResult<HttpResponse> {
    info!("fetch_model query={:?}", info);

    match info.result_type {
        ResultType::Heightmap => {
            Ok(HttpResponse::NotImplemented().finish())
        },
        ResultType::Model => {
            Ok(HttpResponse::NotImplemented().finish())
        },
    }
}

#[post("/new")]
async fn create_new(session: Session) -> AWResult<HttpResponse> {
    session.renew();
    let mut rng = rand::thread_rng();
    let id: u32 = rng.gen();
    session.insert(REQUEST_ID_COOKIE_KEY, id)?;
    // session.set(REQUEST_ID_COOKIE_KEY, id)?;
    Ok(HttpResponse::Ok().finish())
}

#[post("/upload")]
async fn create_upload(
    session: Session,
    redis: web::Data<Addr<RedisActor>>,
    mut payload: mp::Multipart
) -> AWResult<HttpResponse> {
    info!("create_upload");

    let id = request_cookie(session)?;
    debug!("request-id: {}", id);

    while let Ok(Some(mut field)) = payload.try_next().await {
        let _content_type = validate_mime_type(field.content_type())
            .ok_or_else(|| actix_web::error::ErrorUnsupportedMediaType("Unknown image type"))?;

        let data = read_byte_chunks(&mut field).await;
        let _resp = redis.send(Command(resp_array!["SETEX", image_key(id), IMAGE_EXPIRATION_SECONDS.to_string(), data])).await;
    }

    Ok(HttpResponse::Ok().finish())
}

#[post("/start")]
async fn create_start(
    session: Session,
    redis: web::Data<Addr<RedisActor>>,
    // storage: Storage,
    // pool: web::Data<bb8::Pool<bb8_lapin::LapinConnectionManager>>,
) -> AWResult<HttpResponse> {
    info!("create_start");

    let id = request_cookie(session)?;
    debug!("request-id: {}", id);

    let resp1 = redis.send(Command(resp_array![
        "XADD",
        FABSEAL_SUBMISSION_QUEUE,
        "MAXLEN",
        "~",
        FABSEAL_SUBMISSION_QUEUE_LIMIT.to_string(),
        "*",
        "request_id", &id.as_bytes()[..]
        ]))
        .await
        .map_err(|e|
            actix_web::error::ErrorInternalServerError( format!("{}", e) )
        )?
        .map_err(|e|
            actix_web::error::ErrorInternalServerError( format!("{}", e) )
        )?;
    match resp1 {
        RespValue::Error(e) => {
            error!("Redis error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        },
        RespValue::BulkString(_) => {
            Ok(HttpResponse::Accepted().finish())
        },
        _ => {
            error!("Unexpected Redis response: {:?}", resp1);
            Ok(HttpResponse::InternalServerError().finish())
        },
    }
}


#[get("/result")]
async fn create_result(
    session: Session,
    redis: web::Data<Addr<RedisActor>>,
    info: web::Query<ResultRequestInfo>
) -> AWResult<HttpResponse> {
    info!("create_result query={:?}", info);
    // Ok(HttpResponse::Processing().finish())

    match info.result_type {
        ResultType::Heightmap => {
            return Ok(HttpResponse::NotImplemented().finish())
        },
        ResultType::Model => {}
    }

    let id = request_cookie(session)?;

    let comm = Command(resp_array!["GET", result_key(id)]);
    debug!("redis command: {:?}", comm);
    // let resp = redis.send(comm).await??;
    let resp = redis.send(comm).await
        .map_err(|e|
            actix_web::error::ErrorInternalServerError( format!("{}", e) )
        )?
        .map_err(|e|
            actix_web::error::ErrorInternalServerError( format!("{}", e) )
        )?;

    let response_data = convert_bytes_response(resp)?;
    Ok(HttpResponse::Ok()
        .content_type("model/stl")
        .body(response_data))
}

#[post("/finish")]
async fn create_finish() -> AWResult<HttpResponse> {
    info!("create_finish");

    Ok(HttpResponse::NotImplemented().finish())
}

pub(crate) fn create_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/userupload")
            .service(fetch_model)
    );
    cfg.service(
        web::scope("/create")
            .service(create_new)
            .service(create_upload)
            .service(create_start)
            .service(create_result)
            .service(create_finish)
    );
}