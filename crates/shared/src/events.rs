use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub id: Uuid,
    pub ts: DateTime<Utc>,
    pub level: EventLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventLevel {
    Info,
    Warn,
    Error,
}

impl EngineEvent {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            ts: Utc::now(),
            level: EventLevel::Info,
            message: message.into(),
        }
    }
}
