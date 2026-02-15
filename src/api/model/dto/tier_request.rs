use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetTierRequest {
    pub name: String,
}


#[derive(Deserialize)]
pub struct CreateTierRequest {
    pub name: String,
    pub max_tracks : i32
}


