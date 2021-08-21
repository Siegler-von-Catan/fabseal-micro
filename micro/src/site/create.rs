use actix::Addr;
use actix_web::{HttpResponse, Result as AWResult, get, post, web};
use actix_session::Session;
use actix_multipart as mp;
use actix_redis::{Command, RedisActor};
use redis_async::resp_array;

use futures_util::TryStreamExt;

use rand::Rng;

use log::{debug, info};

use fabseal_micro_common::{ImageRequest, StoredImage, input_key, result_key};

use crate::site::rmq_ops::rmq_publish;
use crate::site::types::*;
use crate::site::util::*;

/*

### Retrieving creations

* Endpoint: `GET /api/v1/userupload/public/result?id=<upload_id>&type=model`
  Content-Type: `model/stl` (or glTF, OBJ?)

* Endpoint: `GET /api/v1/userupload/public/result?id=<upload_id>&type=heightmap`
  Content-Type: `image/jpeg`?
*/


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

/*
### New

* Endpoint `POST /api/v1/create/new`
  `200 OK id=<request_id>`

* Endpoint `POST /api/v1/create/upload?id=<request_id>`
  Content-Type: `image/jpeg`

* Endpoint `POST /api/v1/create/start?id=<request_id>`
  Content-Type: `application/json` (constaining request settings)

* Endpoint `GET /api/v1/create/result?id=<request_id>&type=heightmap`
  Content-Type: `image/jpeg`?

* Endpoint `GET /api/v1/create/result?id=<request_id>&type=model`
  Content-Type: `model/stl` (or glTF, OBJ?)

* Endpoint `POST /api/v1/create/finish?id=<request_id>`
  `200 OK id=<upload_id>`
*/


#[post("/new")]
async fn create_new(session: Session) -> AWResult<HttpResponse> {
    session.renew();
    let mut rng = rand::thread_rng();
    let id: u32 = rng.gen();
    // session.insert(REQUEST_ID_COOKIE_KEY, id)?;
    session.set(REQUEST_ID_COOKIE_KEY, id)?;
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
        let content_type = validate_mime_type(field.content_type())
            .ok_or_else(|| actix_web::error::ErrorUnsupportedMediaType("Unknown image type"))?;

        let data = read_byte_chunks(&mut field).await;

        let si = StoredImage { image_type: content_type, image: data };
        let si_vec = serde_json::to_vec(&si)?;
        let _resp = redis.send(Command(resp_array!["SET", input_key(id), si_vec])).await??;
    }

    Ok(HttpResponse::Ok().finish())
}

#[post("/start")]
async fn create_start(
    session: Session,
    redis: web::Data<Addr<RedisActor>>,
    // storage: Storage,
    pool: web::Data<bb8::Pool<bb8_lapin::LapinConnectionManager>>,
) -> AWResult<HttpResponse> {
    info!("create_start");

    let id = request_cookie(session)?;
    debug!("request-id: {}", id);

    let resp = redis.send(Command(resp_array!["GET", input_key(id)])).await??;
    let data = convert_bytes_response(resp)?;

    let conn = pool.get().await.unwrap();

    let si = serde_json::from_slice(data.as_ref())?;

    let ireq = ImageRequest { request_id: id, image: si };
    let ireq_vec = serde_json::to_vec(&ireq)?;

    rmq_publish(conn,  ireq_vec).await.map_err(|e|
        actix_web::error::ErrorInternalServerError( format!("{}", e) )
    )?;

    Ok(HttpResponse::Accepted().finish())
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
    let resp = redis.send(comm).await??;

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