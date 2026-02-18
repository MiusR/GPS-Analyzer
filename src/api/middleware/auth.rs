use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use tower_cookies::Cookies;

use crate::{api::{model::auth::claims::AccessClaims, state::AppState}, errors::app_error::AppError};


// An Axum extractor that validates the `Authorization: Bearer <token>` header
// and injects the decoded `AccessClaims` into handler arguments.
//
// Usage in a handler:
// ```rust
// async fn my_handler(
//     AuthenticatedUser(claims): AuthenticatedUser,
// ) { ... }
// ```
const ACCESS_TOKEN_COOKIE: &str = "access_token";

pub struct AuthenticatedUser(pub AccessClaims);


impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookies using the Cookies extractor
        let cookies = parts
            .extensions
            .get::<Cookies>()
            .ok_or(AppError::invalid_token())?;

        // Get the access token from the cookie
        let token = cookies
            .get(ACCESS_TOKEN_COOKIE)
            .ok_or(AppError::invalid_token())?
            .value().to_string();

        // Verify the token
        let claims = state.get_jwt_service().verify_access_token(&token)?;

        Ok(AuthenticatedUser(claims))
    }
}

// ─── Helper for Bearer token fallback (optional) ──────────────────────────────

/// If you want to support BOTH cookies AND Bearer tokens (for API clients),
/// you can use this extractor instead:
pub struct AuthenticatedUserFlexible(pub AccessClaims);


impl FromRequestParts<AppState> for AuthenticatedUserFlexible {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try cookie first
        if let Some(cookies) = parts.extensions.get::<Cookies>() {
            if let Some(cookie) = cookies.get(ACCESS_TOKEN_COOKIE) {
                if let Ok(claims) = state.get_jwt_service().verify_access_token(cookie.value()) {
                    return Ok(AuthenticatedUserFlexible(claims));
                }
            }
        }

        // Fall back to Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::invalid_token())?;

        let claims = state.get_jwt_service().verify_access_token(auth_header)?;
        Ok(AuthenticatedUserFlexible(claims))
    }
}
