use haste_jwt::{ProjectId, TenantId, VersionId};
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Subscription {
    pub id: String,
    pub version_id: VersionId,
    pub tenant: TenantId,
    pub project: ProjectId,
    pub status: String,
    pub reason: String,
    pub critieria: String,
    // Where to send the notifications
    pub channel_type: String,
    pub channel_endpoint: Option<String>,
    pub channel_payload: Option<String>,
    pub channel_headers: Option<serde_json::Value>,

    // Tracking fields
    pub last_event_sequence: i64,
}

pub struct SubscriptionSearchClaims {}

pub struct CreateSubscription {
    pub id: String,
    pub version_id: VersionId,
    pub tenant: TenantId,
    pub project: ProjectId,
    pub status: String,
    pub reason: String,
    pub critieria: String,
    // Where to send the notifications
    pub channel_type: String,
    pub channel_endpoint: Option<String>,
    pub channel_payload: Option<String>,
    pub channel_headers: Option<serde_json::Value>,
}
