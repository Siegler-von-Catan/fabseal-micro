use std::{
    array::TryFromSliceError,
    convert::{TryFrom, TryInto},
    fmt,
};

use serde::{Deserialize, Serialize};

use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub enum TryFromRequestIdError {
    LengthError(usize),
    SliceError(TryFromSliceError),
}

impl fmt::Display for TryFromRequestIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryFromRequestIdError::LengthError(sz) => {
                write!(f, "unsufficient length {}, expected at least 4 bytes", sz)
            }
            TryFromRequestIdError::SliceError(tfse) => {
                write!(f, "error converting from slice: {}", tfse)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RequestId(u32);

impl RequestId {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let id: u32 = rng.gen();
        RequestId(id)
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn from_bytes(v: &[u8]) -> Result<Self, TryFromRequestIdError> {
        RequestId::try_from(v)
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<u32> for RequestId {
    fn from(id: u32) -> RequestId {
        RequestId(id)
    }
}

impl TryFrom<&[u8]> for RequestId {
    type Error = TryFromRequestIdError;

    fn try_from(v: &[u8]) -> Result<RequestId, Self::Error> {
        if v.len() != 4 {
            return Err(TryFromRequestIdError::LengthError(v.len()));
        }
        let (int_bytes, _) = v.split_at(std::mem::size_of::<u32>());
        let ci = int_bytes
            .try_into()
            .map_err(TryFromRequestIdError::SliceError);
        Ok(RequestId(u32::from_le_bytes(ci?)))
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}

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

pub fn processed_image_key(request_id: RequestId) -> String {
    const REDIS_NAMESPACE_PROCESSED_IMAGE: &str = "processed_image";
    format!(
        "{}:{}:{}",
        REDIS_NAMESPACE, REDIS_NAMESPACE_PROCESSED_IMAGE, request_id
    )
}
