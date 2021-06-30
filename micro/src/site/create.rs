use actix_web::{get, post, web,  HttpResponse, Result as AWResult};

use serde::{Deserialize, Serialize};


use lapin::{Result as LAResult, protocol::basic::AMQPProperties};
use amq_protocol_types::{FieldTable};

use log::info;

#[derive(Deserialize, Debug)]
struct RequestInfo {
    request_id: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum ResultType {
    Model,
    Heightmap
}


#[derive(Deserialize, Debug)]
struct ResultRequestInfo {
    request_id: i32,
    #[serde(rename = "type")]
    result_type: ResultType,
}

#[derive(Serialize, Deserialize, Debug)]
struct BBReq {
    request_id: i32,
    image_id: i32,
}

const FABSEAL_QUEUE: &'static str = "fabseal";
const FABSEAL_EXCHANGE: &'static str = "";

pub async fn rmq_declare(
    pool: bb8::Pool<bb8_lapin::LapinConnectionManager>
) -> LAResult<()> {
    let conn = pool.get().await.unwrap();
    let channel = conn.create_channel()
        .await
        .unwrap();

    info!("CONNECTED");

    let queue = channel
        .queue_declare(
            FABSEAL_QUEUE,
            lapin::options::QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("Declared queue {:?}", queue);

    Ok(())
}


lazy_static::lazy_static! {
    static ref PROPS: AMQPProperties = {
        lapin::BasicProperties::default()
            .with_content_type("application/json".into())
    };
    static ref PUBLISH_OPTIONS: lapin::options::BasicPublishOptions = {
        lapin::options::BasicPublishOptions::default()
    };
}

async fn rmq_publish(
    conn: bb8::PooledConnection<'_, bb8_lapin::LapinConnectionManager>,
    data: &BBReq,
) -> LAResult<()> {
    let channel = conn.create_channel()
        .await
        .unwrap();



    let payload = serde_json::to_vec(&data).unwrap();
    let confirm = channel.basic_publish(
            FABSEAL_EXCHANGE,
            FABSEAL_QUEUE,
            PUBLISH_OPTIONS.clone(),
            payload,
            PROPS.clone()
        )
        .await?
        .await?;

    debug_assert_eq!(confirm, lapin::publisher_confirm::Confirmation::NotRequested);

    Ok(())
}

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
            Ok(HttpResponse::MethodNotAllowed().finish())
        },
        ResultType::Model => {
            Ok(HttpResponse::MethodNotAllowed().finish())
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
async fn create_new() -> AWResult<HttpResponse> {
    Ok(HttpResponse::MethodNotAllowed().finish())
}


#[post("/upload")]
async fn create_upload(
    info: web::Query<RequestInfo>
) -> AWResult<HttpResponse> {
    info!("create_upload query={:?}", info);

    Ok(HttpResponse::MethodNotAllowed().finish())
}

#[post("/start")]
async fn create_start(
    pool: web::Data<bb8::Pool<bb8_lapin::LapinConnectionManager>>,
    info: web::Query<RequestInfo>
) -> AWResult<HttpResponse> {
    // info!("create_start query={:?}", info);

    let conn = pool.get().await.unwrap();

    let r = BBReq { request_id: 123, image_id: 456 };
    rmq_publish(conn,  &r).await.map_err(|e|
        actix_web::error::ErrorInternalServerError( format!("{}", e) )
    )?;

    Ok(HttpResponse::MethodNotAllowed().finish())
}


#[get("/result")]
async fn create_result(
    info: web::Query<ResultRequestInfo>
) -> AWResult<HttpResponse> {
    info!("create_result query={:?}", info);

    match info.result_type {
        ResultType::Heightmap => {
            Ok(HttpResponse::MethodNotAllowed().finish())
        },
        ResultType::Model => {
            Ok(HttpResponse::MethodNotAllowed().finish())
        },
    }
}

#[post("/finish")]
async fn create_finish(
    info: web::Query<RequestInfo>
) -> AWResult<HttpResponse> {
    info!("create_finish query={:?}", info);

    Ok(HttpResponse::MethodNotAllowed().finish())
}

pub fn create_service(cfg: &mut web::ServiceConfig) {
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