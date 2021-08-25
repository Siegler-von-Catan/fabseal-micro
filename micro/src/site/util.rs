use actix_web::{Result as AWResult};
use actix_session::Session;
use actix_multipart as mp;

use futures_util::{stream::{StreamExt}};

use redis_async::resp::RespValue;

use log::{debug, error};

use fabseal_micro_common::{ImageType, RequestId};

pub(crate) const REQUEST_ID_COOKIE_KEY: &str = "request-id";

pub(crate) fn request_cookie(session: &Session)
-> AWResult<RequestId> {
    let raw_id = session.get::<u32>(REQUEST_ID_COOKIE_KEY)?
        .ok_or_else(|| actix_web::error::ErrorForbidden("Session cookie is required for upload"))?;
    Ok(RequestId::create(raw_id))
}

const UPLOAD_LIMIT: usize = 8 * 1024 * 1024;

pub(crate) async fn read_byte_chunks(
   field: &mut mp::Field
) -> AWResult<Vec<u8>> {
    let mut data: Vec<u8> = Vec::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk?;
        if data.len() + chunk.len() > UPLOAD_LIMIT {
            return Err(actix_web::error::ErrorPayloadTooLarge("Upload limit exceeded"));
        }
        data.extend_from_slice(chunk.as_ref());
    }
    Ok(data)
}

pub(crate) fn validate_mime_type(
   content_type: &mime::Mime
) -> AWResult<ImageType> {
    if content_type.type_() != mime::IMAGE {
        return Err(actix_web::error::ErrorUnsupportedMediaType("Only images are supported"));
    }

    match content_type.subtype() {
        mime::JPEG => {
            debug!("jpeg!");
            Ok(ImageType::JPEG)
        },
        mime::PNG => {
            debug!("png!");
            Ok(ImageType::PNG)
        },
        _ => {
            debug!("unknown subtype: {}", content_type);
            Err(actix_web::error::ErrorUnsupportedMediaType("Unknown image subtype"))
        },
    }
}

pub(crate) fn convert_bytes_response(
    resp: RespValue
) -> AWResult<Vec<u8>> {
    match resp {
        RespValue::Nil => {
            Err(actix_web::error::ErrorNotFound("Not found"))
        },
        RespValue::BulkString(data) => {
            Ok(data)
        },
        _ => {
            Err(actix_web::error::ErrorInternalServerError("Not found"))
        }
    }
}

pub(crate) fn redis_error<T>(
    cmd: &str
)
-> impl FnOnce(T) -> actix_web::error::Error + '_
where
    T: std::error::Error
{
    move |e| {
        error!("Error while running {}: {}", cmd, e);
        actix_web::error::ErrorInternalServerError( "Redis error" )
    }
}