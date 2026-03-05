use axum::{Json, extract::State, response::IntoResponse};

use crate::{api::{middleware::auth::AuthenticatedUser, model::dto::tier_request::GetTierRequest, state::AppState}, errors::app_error::AppError};

/*
    API endpoint for requesting data about a specific tier
*/
pub async fn get_tier_info(
    AuthenticatedUser(_): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload) : Json<GetTierRequest>
) -> Result<impl IntoResponse, AppError> {
    let tier =  state.get_tier_service().get_tier_by_name(&payload.name).await?;
    Ok(Json::from(tier))
}

