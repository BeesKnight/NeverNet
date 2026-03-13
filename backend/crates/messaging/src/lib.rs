pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";

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

pub mod subjects {
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
