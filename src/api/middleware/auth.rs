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
