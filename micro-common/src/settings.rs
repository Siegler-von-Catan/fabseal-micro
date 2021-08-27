use serde::Deserialize;

const RESULT_EXPIRATION_SECONDS: u32 = 10 * 60;
const IMAGE_EXPIRATION_SECONDS: u32 = 10 * 60;
const SESSION_TTL_SECONDS: u32 = 30 * 60;

const FABSEAL_SUBMISSION_QUEUE_LIMIT: u32 = 50;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RedisSettings {
    pub address: String,
    pub db_id: Option<i64>,
    pub password: Option<String>,
}

impl Default for RedisSettings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:6379".to_string(),
            db_id: None,
            password: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Limits {
    pub queue_limit: u32,
    pub image_ttl: u32,
    pub result_ttl: u32,
    pub session_ttl: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            queue_limit: FABSEAL_SUBMISSION_QUEUE_LIMIT,
            image_ttl: IMAGE_EXPIRATION_SECONDS,
            result_ttl: RESULT_EXPIRATION_SECONDS,
            session_ttl: SESSION_TTL_SECONDS,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct HttpSettings {
    pub endpoint: String,
    pub cookie_domain: Option<String>,
}

impl Default for HttpSettings {
    fn default() -> Self {
        Self {
            endpoint: "127.0.0.1:8080".to_string(),
            cookie_domain: None,
        }
    }
}