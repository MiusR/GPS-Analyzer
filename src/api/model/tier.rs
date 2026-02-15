use serde::Serialize;
use uuid::Uuid;


#[derive(Clone, Debug, Serialize)]
pub struct Tier {
    pub id : Uuid,
    pub name : String,
    pub max_tracks : i32
}