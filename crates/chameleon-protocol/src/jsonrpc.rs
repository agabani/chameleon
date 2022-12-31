use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Error {
    #[serde(rename = "code")]
    pub code: i64,

    #[serde(rename = "message")]
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Frame<T, U> {
    pub jsonrpc: String,

    #[serde(flatten)]
    pub type_: FrameType<T, U>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FrameType<T, U> {
    Request(RequestFrame<T>),
    Response(ResponseFrame<U>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RequestFrame<T> {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResponseFrame<T> {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    #[serde(rename = "result", skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    #[serde(rename = "error", skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

impl<T, U> Frame<T, U> {
    pub fn parse_error() -> Frame<T, U> {
        Frame {
            jsonrpc: "2.0".to_string(),
            type_: FrameType::Response(ResponseFrame {
                id: None,
                result: None,
                error: Some(Error {
                    code: -32700,
                    message: "Parse error".to_string(),
                }),
            }),
        }
    }
}

impl<'a, T, U> Frame<T, U> {
    pub fn try_from_str(s: &'a str) -> Result<Frame<T, U>, serde_json::Error>
    where
        T: serde::de::Deserialize<'a>,
        U: serde::de::Deserialize<'a>,
    {
        serde_json::from_str(s)
    }
}

impl<T, U> Frame<T, U>
where
    T: serde::Serialize,
    U: serde::Serialize,
{
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
