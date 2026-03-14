use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";
pub const DOMAIN_EVENTS_STREAM: &str = "EVENTDESIGN_DOMAIN_EVENTS";
pub const PROJECTION_CONSUMER: &str = "projection-worker";
pub const EXPORT_CONSUMER: &str = "export-worker";

#[derive(Debug, Clone)]
pub struct MessagingConfig {
    pub nats_url: String,
}

impl MessagingConfig {
    pub fn from_env() -> Self {
        Self {
            nats_url: std::env::var("NATS_URL").unwrap_or_else(|_| DEFAULT_NATS_URL.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventEnvelope {
    pub id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_version: i32,
    pub occurred_at: DateTime<Utc>,
    pub payload: Value,
}

pub fn subject_for_event_type(event_type: &str) -> String {
    format!("eventdesign.{event_type}")
}

pub mod subjects {
    pub const ALL: &str = "eventdesign.>";
    pub const USER_REGISTERED: &str = "eventdesign.user.registered";
    pub const USER_LOGGED_IN: &str = "eventdesign.user.logged_in";
    pub const CATEGORY_CREATED: &str = "eventdesign.category.created";
    pub const CATEGORY_UPDATED: &str = "eventdesign.category.updated";
    pub const CATEGORY_DELETED: &str = "eventdesign.category.deleted";
    pub const EVENT_CREATED: &str = "eventdesign.event.created";
    pub const EVENT_UPDATED: &str = "eventdesign.event.updated";
    pub const EVENT_DELETED: &str = "eventdesign.event.deleted";
    pub const EVENT_STATUS_CHANGED: &str = "eventdesign.event.status_changed";
    pub const EXPORT_REQUESTED: &str = "eventdesign.export.requested";
    pub const EXPORT_STARTED: &str = "eventdesign.export.started";
    pub const EXPORT_COMPLETED: &str = "eventdesign.export.completed";
    pub const EXPORT_FAILED: &str = "eventdesign.export.failed";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_default_messaging_config() {
        unsafe {
            std::env::remove_var("NATS_URL");
        }

        let config = MessagingConfig::from_env();

        assert_eq!(config.nats_url, DEFAULT_NATS_URL);
    }

    #[test]
    fn builds_subject_name_from_event_type() {
        assert_eq!(
            subject_for_event_type("category.created"),
            "eventdesign.category.created"
        );
    }
}
