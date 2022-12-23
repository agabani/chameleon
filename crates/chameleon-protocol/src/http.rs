use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageRequest {
    #[serde(rename = "content")]
    pub content: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum TelemetryLevel {
    #[serde(rename = "trace")]
    Trace,

    #[serde(rename = "debug")]
    Debug,

    #[serde(rename = "info")]
    Info,

    #[serde(rename = "warn")]
    Warn,

    #[serde(rename = "error")]
    Error,
}

impl AsRef<str> for TelemetryLevel {
    fn as_ref(&self) -> &str {
        match self {
            TelemetryLevel::Trace => "trace",
            TelemetryLevel::Debug => "debug",
            TelemetryLevel::Info => "info",
            TelemetryLevel::Warn => "warn",
            TelemetryLevel::Error => "error",
        }
    }
}
