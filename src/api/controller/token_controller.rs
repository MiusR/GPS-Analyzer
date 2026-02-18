use axum::{Json, extract::State};
use chrono::{Duration, Utc};

use crate::{api::{model::dto::jwt_request::{RefreshRequest, RevokeRequest, TokenResponse}, service::jwt_service::{ACCESS_TOKEN_TTL_SECS, REFRESH_TOKEN_TTL_SECS}, state::AppState}, errors::app_error::AppError};


pub async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {

    let claims = state.get_jwt_service().verify_refresh_token(&body.refresh_token)?;

    let _record = state
        .validate_refresh_token(&claims.jti).await
        .ok_or(AppError::token_revoked())?;

    state.revoke_refresh_token(&claims.jti);


    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| AppError::invalid_token())?;
    let user = state.get_user_service().get_user_by_uuid(&user_id).await?;

    tracing::debug!("Rotating refresh token for user: {}", user.get_uuid());

    let access_token = state.get_jwt_service().issue_access_token(&user)?;

    let (refresh_token, new_jti) = state.get_jwt_service().issue_refresh_token(&user)?;
    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS);
    state.store_refresh_token(user.get_uuid().clone(), new_jti, expires_at);

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: ACCESS_TOKEN_TTL_SECS as u64,
    }))
}


pub async fn revoke_token(
    State(state): State<AppState>,
    Json(body): Json<RevokeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // We still verify the JWT signature so we get the JTI
    let claims = state.get_jwt_service().verify_refresh_token(&body.refresh_token)?;
    state.revoke_refresh_token(&claims.jti);

    tracing::debug!("Revoked refresh token jti={}", claims.jti);
    Ok(Json(serde_json::json!({ "message": "Token revoked successfully" })))
}


pub async fn logout_all(
    State(state): State<AppState>,
    Json(body): Json<RevokeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims = state.get_jwt_service().verify_refresh_token(&body.refresh_token)?;
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| AppError::invalid_token())?;

    state.revoke_all_user_tokens(&user_id);

    tracing::debug!("Revoked all tokens for user: {}", user_id);
    Ok(Json(serde_json::json!({ "message": "Logged out from all devices" })))
}
