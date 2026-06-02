use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub id: Uuid,
    #[serde(default)]
    pub seq: u64,
    pub ts: DateTime<Utc>,
    #[serde(default)]
    pub level: EventLevel,
    #[serde(default)]
    pub kind: EventKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    #[default]
    Info,
    Warn,
    Error,
    Heartbeat,
    DryRunOpportunity,
    DryRunExecuted,
}

impl EngineEvent {
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(EventLevel::Info, EventKind::Info, message)
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(EventLevel::Warn, EventKind::Warn, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(EventLevel::Error, EventKind::Error, message)
    }

    pub fn heartbeat(message: impl Into<String>) -> Self {
        Self::new(EventLevel::Info, EventKind::Heartbeat, message)
    }

    pub fn dry_run_opportunity(message: impl Into<String>) -> Self {
        Self::new(EventLevel::Info, EventKind::DryRunOpportunity, message)
    }

    pub fn dry_run_executed(message: impl Into<String>) -> Self {
        Self::new(EventLevel::Info, EventKind::DryRunExecuted, message)
    }

    fn new(level: EventLevel, kind: EventKind, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            seq: 0,
            ts: Utc::now(),
            level,
            kind,
            message: message.into(),
        }
    }
}
