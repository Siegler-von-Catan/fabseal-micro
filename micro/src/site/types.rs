use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub(crate) struct RequestInfo {
    request_id: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "lowercase"))]
pub(crate) enum ResultType {
    Model,
    Heightmap
}


#[derive(Deserialize, Debug)]
pub(crate) struct ResultRequestInfo {
    request_id: i32,
    #[serde(rename = "type")]
    pub(crate) result_type: ResultType,
}

