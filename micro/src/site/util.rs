use actix_web::{Result as AWResult};
use actix_session::Session;
use actix_multipart as mp;

use futures_util::{stream::{StreamExt}};

use redis_async::resp::RespValue;

use log::debug;

use fabseal_micro_common::{ImageType, RequestId};

pub(crate) const REQUEST_ID_COOKIE_KEY: &str = "request-id";

pub(crate) fn request_cookie(session: Session)
-> AWResult<RequestId> {
    let raw_id = session.get::<u32>(REQUEST_ID_COOKIE_KEY)?
        .ok_or_else(|| actix_web::error::ErrorForbidden("Session cookie is required for upload"))?;
    Ok(RequestId::create(raw_id))
}

pub(crate) async fn read_byte_chunks(
   field: &mut mp::Field
) -> Vec<u8> {
   let mut data: Vec<u8> = Vec::new();
   while let Some(Ok(chunk)) = field.next().await {
       data.extend_from_slice(chunk.as_ref());
   }
   data
}

pub(crate) fn validate_mime_type(
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
