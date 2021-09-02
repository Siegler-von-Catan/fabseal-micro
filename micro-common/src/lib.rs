use serde::{Deserialize, Serialize};

pub mod request_id;
pub use request_id::RequestId;

pub mod settings;

#[derive(Serialize, Deserialize, Debug)]
pub enum ImageType {
    PNG,
    JPEG,
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

pub const FABSEAL_SUBMISSION_QUEUE: &str = "fs_submission";
pub const FABSEAL_SUBMISSION_CONSUMER_GROUP: &str = "fs_submission_group";

const REDIS_NAMESPACE: &str = "fsdata_v1";

pub fn result_key(request_id: RequestId) -> String {
    const REDIS_NAMESPACE_RESULT: &str = "result";
    format!(
        "{}:{}:{}",
        REDIS_NAMESPACE, REDIS_NAMESPACE_RESULT, request_id
    )
}

pub fn input_key(request_id: RequestId) -> String {
    const REDIS_NAMESPACE_INPUT: &str = "input";
    format!(
        "{}:{}:{}",
        REDIS_NAMESPACE, REDIS_NAMESPACE_INPUT, request_id
    )
}

pub fn image_key(request_id: RequestId) -> String {
    const REDIS_NAMESPACE_IMAGE: &str = "image";
    format!(
        "{}:{}:{}",
        REDIS_NAMESPACE, REDIS_NAMESPACE_IMAGE, request_id
    )
}
