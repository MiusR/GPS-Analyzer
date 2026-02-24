use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RacingEvent {
    pub uuid : Uuid,
    pub event_name : String,
    pub created_at: DateTime<Utc>,
}
