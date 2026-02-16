use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tier {
    pub uuid : Uuid,
    pub name : String,
    pub max_tracks : i32
}