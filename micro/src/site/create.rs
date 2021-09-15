use actix::Addr;
use actix_multipart as mp;
use actix_redis::{Command, RedisActor, RespValue};
use actix_session::Session;
use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    post, web, HttpResponse, Result as AWResult,
};

use futures_util::TryStreamExt;

use log::{debug, error, info, trace};

use fabseal_micro_common::*;
use redis_async::resp_array;

use crate::{
    prepare_image::run,
    settings::Settings,
    site::{types::*, util::*},
};

#[get("/public/result")]
async fn fetch_model(info: web::Query<ResultRequestInfo>) -> AWResult<HttpResponse> {
    info!("fetch_model query={:?}", info);

    match info.result_type {
        ResultType::Heightmap => Ok(HttpResponse::NotImplemented().finish()),
        ResultType::Model => Ok(HttpResponse::NotImplemented().finish()),
    }
}


#[post("/upload")]
async fn create_upload(
    session: Session,
    redis: web::Data<Addr<RedisActor>>,
    settings: web::Data<Settings>,
    mut payload: mp::Multipart,
) -> AWResult<HttpResponse> {
    info!("create_upload");

    let id = request_cookie(&session)?;
    debug!("request-id: {}", id);

    while let Some(mut field) = payload.try_next().await? {
        let content_type = validate_mime_type(field.content_type())?;
        trace!("received image type: {:?}", content_type);

        let data = read_byte_chunks(&mut field).await?;

        let resp = redis
            .send(Command(resp_array![
                "SETEX",
                image_key(id),
                settings.limits.image_ttl.to_string(),
                data.as_slice()
            ]))
            .await
            .map_err(redis_error("SETEX"))?
            .map_err(redis_error("SETEX"))?;

        debug_assert_eq!(resp, RespValue::SimpleString("OK".to_string()));

        let processed = actix_web::rt::task::spawn_blocking(move || run(&data))
            .await
            .unwrap()
            .map_err(|e| {
                error!("error processing image: {}", e);
                actix_web::error::ErrorInternalServerError("processing error")
            })?;

        let resp = redis
            .send(Command(resp_array![
                "SETEX",
                processed_image_key(id),
                settings.limits.image_ttl.to_string(),
                processed.as_slice()
            ]))
            .await
            .map_err(redis_error("SETEX"))?
            .map_err(redis_error("SETEX"))?;

        debug_assert_eq!(resp, RespValue::SimpleString("OK".to_string()));
    }

    Ok(HttpResponse::Ok().finish())
}

#[post("/start")]
async fn create_start(
    session: Session,
    redis: web::Data<Addr<RedisActor>>,
    settings: web::Data<Settings>,
) -> AWResult<HttpResponse> {
    info!("create_start");

    let id = request_cookie(&session)?;
    debug!("request-id: {}", id);

    let resp1 = redis
        .send(Command(resp_array![
            "XADD",
            FABSEAL_SUBMISSION_QUEUE,
            "MAXLEN",
            "~",
            settings.limits.queue_limit.to_string(),
            "*",
            "request_id",
            &id.as_bytes()[..]
        ]))
        .await
        .map_err(redis_error("XADD"))?
        .map_err(redis_error("XADD"))?;
    match resp1 {
        RespValue::Error(e) => {
            error!("Redis error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
        RespValue::BulkString(_) => Ok(HttpResponse::Accepted().finish()),
        _ => {
            error!("Unexpected Redis response: {:?}", resp1);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}

#[get("/result")]
async fn create_result(
    session: Session,
    redis: web::Data<Addr<RedisActor>>,
    info: web::Query<ResultRequestInfo>,
) -> AWResult<HttpResponse> {
    info!("create_result query={:?}", info);
    // Ok(HttpResponse::Processing().finish())

    let id = request_cookie(&session)?;

    let mut response_builder = HttpResponse::Ok();

    let key_function: fn(RequestId) -> String = match info.result_type {
        ResultType::Heightmap => {
            response_builder.content_type("image/jpeg");
            processed_image_key
        }
        ResultType::Model => {
            let cd = ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![DispositionParam::Filename(format!("model_{}.stl", id))],
            };
            response_builder.append_header(cd);
            response_builder.content_type("model/stl");
            result_key
        }
    };

    let resp = redis
        .send(Command(resp_array!["GET", key_function(id)]))
        .await
        .map_err(redis_error("GET"))?
        .map_err(redis_error("GET"))?;

    let response_data = convert_bytes_response(resp)?;

    Ok(response_builder.body(response_data))
}

#[post("/finish")]
async fn create_finish() -> AWResult<HttpResponse> {
    info!("create_finish");

    Ok(HttpResponse::NotImplemented().finish())
}

pub(crate) fn create_service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/userupload").service(fetch_model));
    cfg.service(
        web::scope("/create")
            .service(create_upload)
            .service(create_start)
            .service(create_result)
            .service(create_finish),
    );
}
