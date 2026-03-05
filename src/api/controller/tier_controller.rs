use axum::{Json, extract::State, response::IntoResponse};

use crate::api::{middleware::auth::AuthenticatedUser, model::dto::tier_request::GetTierRequest, state::AppState};

/*
    API endpoint for requesting data about a specific tier
*/
pub async fn get_tier_info(
    AuthenticatedUser(_): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload) : Json<GetTierRequest>
) -> impl IntoResponse {
    match state.get_tier_service().get_tier_by_name(&payload.name).await {
        Ok(tier) => axum::Json::from(tier).into_response(),
        Err(err) => err.into_response()
    }
}

