use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub enum ImageType {
    PNG,
    JPEG
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredImage {
    pub image_type: ImageType,
    pub image: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageRequest {
    pub request_id: u32,
    pub image: StoredImage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestSettings {
    pub start_x: i32,
    pub end_x: i32,
    pub start_y: i32,
    pub end_y: i32,
    pub is_inverted: bool,
    pub is_low_quality: bool,
}

pub const FABSEAL_QUEUE: &str = "fabseal";

pub const FABSEAL_EXCHANGE: &str = "";

pub const RESULT_EXPIRATION_SECONDS: usize = 10*60;

const REDIS_NAMESPACE: &str = "fsdata_v1";
const REDIS_NAMESPACE_RESULT: &str = "result";
const REDIS_NAMESPACE_INPUT: &str = "input";

pub fn result_key(
    request_id: u32
) -> String {
    format!("{}_{}_{}", REDIS_NAMESPACE, REDIS_NAMESPACE_RESULT, request_id)
}

pub fn input_key(
    request_id: u32
) -> String {
    format!("{}_{}_{}", REDIS_NAMESPACE, REDIS_NAMESPACE_INPUT, request_id)
}
