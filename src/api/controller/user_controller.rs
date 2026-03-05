use axum::{Json, extract::State, response::IntoResponse};
use serde_json::json;

use crate::{api::{middleware::auth::AuthenticatedUser, model::dto::user_request::{DeleteUserRequest, GetUserRequest, UpdateUserRequest}, state::AppState}, errors::{app_error::AppError, service_errors::ServiceError}};


pub async fn get_me(
    AuthenticatedUser(claims): AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {

    // Fetch user from storage
    let user = state
        .get_user_service()
        .get_user_by_uuid(&claims.sub.parse().map_err(|_| AppError::invalid_token())?)
        .await?;

    // Return user info
    Ok(Json(json!({
        "id": user.get_uuid().to_string(),
        "provider": user.get_provider().to_string(),
        "email": user.get_email(),
        "name": user.get_name(),
        "avatar_url": user.get_avatar_url(),
        "created_at": user.get_created_time(),
    })))
}

pub async fn get_user(
    State(state) : State<AppState>,
    Json(payload) : Json<GetUserRequest>
) -> Result<impl IntoResponse, AppError> {
    if let Some(email) = payload.email {
        let user = state.get_user_service().get_user_by_email(&email).await?;
        return Ok(Json::from(user));
    }

    if let Some(uuid) = payload.uuid {
        let user=  state.get_user_service().get_user_by_uuid(&uuid).await?;
        return Ok(Json::from(user));
    }

    Err(AppError::service_error(ServiceError::invalid_data("Request must contain either a valid user uuid or email.")))
}


pub async fn update_user(
    State(state) : State<AppState>,
    Json(payload) : Json<UpdateUserRequest>
) -> Result<impl IntoResponse, AppError> {
    let user = state.get_user_service().update_user(&payload.uuid, payload.name, payload.email, payload.tier, payload.avatar_url).await?;
    Ok(Json::from(user))
}

pub async fn delete_user(
    State(state) : State<AppState>,
    Json(payload) : Json<DeleteUserRequest>
) -> Result<impl IntoResponse, AppError> {
    let user = state.get_user_service().delete_user(&payload.uuid).await?;
    Ok(Json::from(user))
}

