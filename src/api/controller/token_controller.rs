use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{Duration, Utc};
use tower_cookies::Cookies;

use crate::{api::{service::oauth_service::OAuthService, state::{AppState, REFRESH_TOKEN_TTL_SECS}}, errors::app_error::AppError};

const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

pub async fn refresh_token(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let refresh_token_str = cookies
        .get(REFRESH_TOKEN_COOKIE)
        .ok_or(AppError::invalid_token())?
        .value()
        .to_string();

    let claims = state.get_jwt_service().verify_refresh_token(&refresh_token_str)?;
    
    let _record = state.get_jwt_service()
        .validate_refresh_token(&claims.jti).await
        .ok_or(AppError::token_revoked())?;

    state.get_jwt_service().revoke_refresh_token(&claims.jti).await?;


    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| AppError::invalid_token())?;
    let user = state.get_user_service().get_user_by_uuid(&user_id).await?;

    tracing::debug!("Rotating refresh token for user: {}", user.get_uuid());

    let access_token = state.get_jwt_service().issue_access_token(&user)?;

    let (refresh_token, new_jti) = state.get_jwt_service().issue_refresh_token(&user)?;
    let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS);
    state.get_jwt_service().store_refresh_token(user.get_uuid().clone(), new_jti, expires_at).await?;

    // TODO save to redis if refresh happens and check if refresh count is higher than
    OAuthService::set_auth_cookies(&cookies, &access_token, &refresh_token);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Tokens refreshed successfully"
        }))
    ).into_response())
}


pub async fn revoke_token(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {

    for cook in cookies.list().iter() {
        tracing::debug!("Name {}, Value {}", cook.name(), cook.value());
    }

    // Get refresh token from cookie
    let refresh_token_str = cookies
        .get(REFRESH_TOKEN_COOKIE)
        .ok_or(AppError::invalid_token())?
        .value()
        .to_string();

    // Verify and get JTI
    let claims = state.get_jwt_service().verify_refresh_token(&refresh_token_str)?;
    state.get_jwt_service().revoke_refresh_token(&claims.jti).await?;

    tracing::info!("Revoked refresh token jti={}", claims.jti);

    // Clear cookies
    OAuthService::clear_auth_cookies(&cookies);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Logged out successfully" }))
    ).into_response())
}


pub async fn logout_all(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    let refresh_token_str = cookies
        .get(REFRESH_TOKEN_COOKIE)
        .ok_or(AppError::invalid_token())?
        .value()
        .to_string();

    let claims = state.get_jwt_service().verify_refresh_token(&refresh_token_str)?;
    let user_id: uuid::Uuid = claims.sub.parse().map_err(|_| AppError::invalid_token())?;

    state.get_jwt_service().revoke_all_user_tokens(&user_id).await?;

    tracing::info!("Revoked all tokens for user: {}", user_id);

    // Clear cookies
    OAuthService::clear_auth_cookies(&cookies);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Logged out from all devices" }))
    ).into_response())
}
