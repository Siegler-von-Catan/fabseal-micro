use std::time::Duration;

use actix_web::{HttpResponse, Result as AWResult, get, post, web::{self}};
use actix_session::Session;
use actix_storage::Storage;
use actix_multipart as mp;

use futures_util::{TryStreamExt, stream::{StreamExt}};


use rand::Rng;
use serde::{Deserialize};

use lapin::{Result as LAResult, protocol::basic::AMQPProperties};
use amq_protocol_types::{FieldTable};

use log::{debug, info};


use fabseal_micro_common::{FABSEAL_EXCHANGE, FABSEAL_QUEUE, ImageRequest, ImageType, StoredImage};


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
    payload: Vec<u8>,
) -> LAResult<()> {
    let channel = conn.create_channel()
        .await
        .unwrap();

    let confirm = channel.basic_publish(
            FABSEAL_EXCHANGE,
            FABSEAL_QUEUE,
            *PUBLISH_OPTIONS,
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

const REQUEST_ID_COOKIE_KEY: &str = "request-id";

#[post("/new")]
async fn create_new(session: Session) -> AWResult<HttpResponse> {
    session.renew();
    let mut rng = rand::thread_rng();
    let id: u32 = rng.gen();
    // session.insert(REQUEST_ID_COOKIE_KEY, id)?;
    session.set(REQUEST_ID_COOKIE_KEY, id)?;
    Ok(HttpResponse::Ok().finish())
}

fn request_cookie(session: Session)
 -> AWResult<u32> {
    session.get::<u32>(REQUEST_ID_COOKIE_KEY)?
        .ok_or_else(|| actix_web::error::ErrorForbidden("Session cookie is required for upload"))
 }

async fn read_byte_chunks(
    field: &mut mp::Field
) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();
    while let Some(Ok(chunk)) = field.next().await {
        data.extend_from_slice(chunk.as_ref());
    }
    data
}

fn validate_mime_type(
    content_type: &mime::Mime
) -> Option<ImageType> {
    if content_type.type_() != mime::IMAGE {
        return None
    }

    match content_type.subtype() {
        mime::JPEG => {
            debug!("jpeg!");
            Some(ImageType::JPEG)
        },
        mime::PNG => {
            debug!("png!");
            Some(ImageType::PNG)
        },
        _ => {
            debug!("unknown subtype: {}", content_type);
            None
        },
    }
}

#[post("/upload")]
async fn create_upload(
    session: Session,
    storage: Storage,
    mut payload: mp::Multipart
) -> AWResult<HttpResponse> {
    info!("create_upload");

    let id = request_cookie(session)?;
    debug!("request-id: {}", id);

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = validate_mime_type(field.content_type())
            .ok_or_else(|| actix_web::error::ErrorUnsupportedMediaType("Unknown image type"))?;

        let data = read_byte_chunks(&mut field).await;

        const UPLOAD_EXPIRY: Duration = Duration::from_secs(3600);

        let si = StoredImage { image_type: content_type, image: data };
        let si_vec = serde_json::to_vec(&si)?;
        storage.set_expiring_bytes(id.to_le_bytes(), si_vec, UPLOAD_EXPIRY).await?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[post("/start")]
async fn create_start(
    session: Session,
    storage: Storage,
    pool: web::Data<bb8::Pool<bb8_lapin::LapinConnectionManager>>,
    info: web::Query<RequestInfo>
) -> AWResult<HttpResponse> {
    info!("create_start query={:?}", info);

    let id = request_cookie(session)?;
    debug!("request-id: {}", id);

    let data = storage
        .get_bytes_ref(id.to_le_bytes())
        .await?
        .ok_or_else(|| actix_web::error::ErrorInternalServerError(
            "Error retrieving internal image data"
        ))?;

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
    _session: Session,
    info: web::Query<ResultRequestInfo>
) -> AWResult<HttpResponse> {
    info!("create_result query={:?}", info);
    // Ok(HttpResponse::Processing().finish())

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