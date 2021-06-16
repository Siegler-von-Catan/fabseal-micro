use actix_web::{get, post, web, App, HttpResponse, HttpServer, Result as AWResult};

use serde::Deserialize;

use log::{info};

#[derive(Deserialize)]
pub struct Info {
    pub name: String,
}

#[derive(Deserialize)]
struct RequestInfo {
    request_id: i32,
}

#[derive(Deserialize)]
struct ResultRequestInfo {
    request_id: i32,
    result_type: String,
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
    if info.result_type == "heightmap" {
        Ok(HttpResponse::MethodNotAllowed().finish())
    } else if info.result_type == "model" {
        Ok(HttpResponse::MethodNotAllowed().finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
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
    Ok(HttpResponse::MethodNotAllowed().finish())
}

#[post("/start")]
async fn create_start(
    info: web::Query<RequestInfo>
) -> AWResult<HttpResponse> {
    Ok(HttpResponse::MethodNotAllowed().finish())
}


#[get("/result")]
async fn create_result(
    info: web::Query<ResultRequestInfo>
) -> AWResult<HttpResponse> {
    if info.result_type == "heightmap" {
        Ok(HttpResponse::MethodNotAllowed().finish())
    } else if info.result_type == "model" {
        Ok(HttpResponse::MethodNotAllowed().finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
    }
}

#[post("/finish")]
async fn create_finish(
    info: web::Query<RequestInfo>
) -> AWResult<HttpResponse> {
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