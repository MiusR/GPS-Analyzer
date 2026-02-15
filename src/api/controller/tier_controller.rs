use axum::{Json, extract::State, response::IntoResponse};

use crate::api::{io::tier_repo::{create_tier, get_tier_by_name}, model::{dto::tier_request::{CreateTierRequest, GetTierRequest}, tier::Tier}, state::ServerState};


/*
    API endpoint for requesting data about a specific tier
*/
pub async fn get_tier_info(
    State(state) : State<ServerState>,
    Json(payload) : Json<GetTierRequest>
) -> impl IntoResponse {
    match get_tier_by_name(&payload.name, state.get_user_db()).await {
        Ok(tier) => axum::Json::from(tier).into_response(),
        Err(err) => err.into_response()
    }
}

/*
    API endpoint for adding data about a specific tier
*/
pub async fn add_tier(
    State(state) : State<ServerState>,
    Json(payload) : Json<CreateTierRequest>
) -> impl IntoResponse {
    match create_tier(&payload.name, payload.max_tracks.clone(), state.get_user_db()).await {
        Ok(uuid) => { 
            let tier = Tier{
                id : uuid,
                max_tracks : payload.max_tracks,
                name : payload.name
            };
            axum::Json::from(tier).into_response()},
        Err(err) => err.into_response()
    }
}