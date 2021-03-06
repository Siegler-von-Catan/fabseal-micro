use actix_multipart as mp;
use actix_session::Session;
use actix_web::Result as AWResult;

use futures_util::stream::StreamExt;

use redis_async::resp::RespValue;

use log::{debug, error, info, warn};

use fabseal_micro_common::{ImageType, RequestId};

pub(crate) const REQUEST_ID_COOKIE_KEY: &str = "request-id";

pub(crate) fn new_request_cookie(session: &Session) -> AWResult<RequestId> {
    let rid: RequestId = RequestId::new();
    let _old_key = session.remove(REQUEST_ID_COOKIE_KEY);
    session.insert(REQUEST_ID_COOKIE_KEY, rid)?;

    debug!("old_key={:?} rid={}", _old_key, rid);

    Ok(rid)
}

pub(crate) fn request_cookie(session: &Session) -> AWResult<RequestId> {
    match session.get::<RequestId>(REQUEST_ID_COOKIE_KEY)? {
        None => {
            new_request_cookie(session)
        }
        Some(rid) => Ok(rid),
    }
}

const UPLOAD_LIMIT: usize = 8 * 1024 * 1024;

pub(crate) async fn read_byte_chunks(field: &mut mp::Field) -> AWResult<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk?;

        debug!("chunk len={}", chunk.len());

        if data.len() + chunk.len() > UPLOAD_LIMIT {
            warn!("Rejected upload: data len={}, chunk len={}, limit={}", data.len(), chunk.len(), UPLOAD_LIMIT);
            return Err(actix_web::error::ErrorPayloadTooLarge(
                "Upload limit exceeded",
            ));
        }
        data.extend_from_slice(chunk.as_ref());
    }
    Ok(data)
}

pub(crate) fn validate_mime_type(content_type: &mime::Mime) -> AWResult<ImageType> {
    if content_type.type_() != mime::IMAGE {
        return Err(actix_web::error::ErrorUnsupportedMediaType(
            "Only images are supported",
        ));
    }

    match content_type.subtype() {
        mime::JPEG => Ok(ImageType::JPEG),
        mime::PNG => Ok(ImageType::PNG),
        _ => {
            info!("unknown subtype: {}", content_type);
            Err(actix_web::error::ErrorUnsupportedMediaType(
                "Unknown image subtype",
            ))
        }
    }
}

pub(crate) fn convert_bytes_response(resp: RespValue) -> AWResult<Vec<u8>> {
    match resp {
        RespValue::BulkString(data) => Ok(data),
        RespValue::Nil => Err(actix_web::error::ErrorNotFound("Not found")),
        RespValue::Error(e) => {
            error!("Redis error: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Redis error"))
        }
        _ => {
            error!("Redis error: Unexpected response");
            Err(actix_web::error::ErrorInternalServerError("Redis error"))
        }
    }
}

pub(crate) fn redis_error<T>(cmd: &str) -> impl FnOnce(T) -> actix_web::error::Error + '_
where
    T: std::error::Error,
{
    move |e| {
        error!("Error while running {}: {}", cmd, e);
        actix_web::error::ErrorInternalServerError("Redis error")
    }
}
