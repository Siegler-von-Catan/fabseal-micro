use serde::{Deserialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "lowercase"))]
pub(crate) enum ResultType {
    Model,
    Heightmap
}

#[derive(Deserialize, Debug)]
pub(crate) struct ResultRequestInfo {
    #[serde(rename = "type")]
    pub(crate) result_type: ResultType,
}

