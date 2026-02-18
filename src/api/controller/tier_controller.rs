use axum::{Json, extract::State, response::IntoResponse};

use crate::api::{model::{dto::tier_request::{CreateTierRequest, GetTierRequest}, tier::Tier}, state::AppState};


/*
    API endpoint for requesting data about a specific tier
*/
pub async fn get_tier_info(
    State(state) : State<AppState>,
    Json(payload) : Json<GetTierRequest>
) -> impl IntoResponse {
    match state.get_tier_service().get_tier_by_name(&payload.name).await {
        Ok(tier) => axum::Json::from(tier).into_response(),
        Err(err) => err.into_response()
    }
}

/*
    API endpoint for adding data about a specific tier
*/
pub async fn add_tier(
    State(state) : State<AppState>,
    Json(payload) : Json<CreateTierRequest>
) -> impl IntoResponse {
    match state.get_tier_service().create_tier(&payload.name, payload.max_tracks.clone()).await {
        Ok(uuid) => { 
            let tier = Tier{
                uuid,
                max_tracks : payload.max_tracks,
                name : payload.name
            };
            axum::Json::from(tier).into_response()},
        Err(err) => err.into_response()
    }
}