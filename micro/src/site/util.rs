use actix_web::{Result as AWResult};
use actix_session::Session;
use actix_multipart as mp;

use futures_util::{stream::{StreamExt}};

use log::debug;

use fabseal_micro_common::ImageType;

pub(crate) const REQUEST_ID_COOKIE_KEY: &str = "request-id";

pub(crate) fn request_cookie(session: Session)
-> AWResult<u32> {
   session.get::<u32>(REQUEST_ID_COOKIE_KEY)?
       .ok_or_else(|| actix_web::error::ErrorForbidden("Session cookie is required for upload"))
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
