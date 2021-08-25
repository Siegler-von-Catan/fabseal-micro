use std::{
    array::TryFromSliceError,
    convert::{TryFrom, TryInto},
    fmt,
};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub struct RequestId(u32);

impl RequestId {
    pub fn create(id: u32) -> RequestId {
        RequestId(id)
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn from_bytes(v: &[u8]) -> Result<RequestId, TryFromSliceError> {
        RequestId::try_from(v)
    }
}

impl TryFrom<&[u8]> for RequestId {
    type Error = TryFromSliceError;
    fn try_from(v: &[u8]) -> Result<RequestId, Self::Error> {
        debug_assert_eq!(v.len(), 4);
        if v.len() != 4 {
            // Err(())
        }
        let (int_bytes, _) = v.split_at(std::mem::size_of::<u32>());
        let ci = int_bytes.try_into();
        Ok(RequestId(u32::from_le_bytes(ci?)))
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rid[{:08X}]", self.0)
    }
}

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

pub const RESULT_EXPIRATION_SECONDS: usize = 10 * 60;
pub const IMAGE_EXPIRATION_SECONDS: usize = 10 * 60;
pub const SESSION_TTL_SECONDS: u32 = 30 * 60;

pub const FABSEAL_SUBMISSION_QUEUE_LIMIT: usize = 50;

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
