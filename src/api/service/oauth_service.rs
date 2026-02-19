use std::sync::Arc;

use axum::{Json, http::{StatusCode}, response::{IntoResponse, Response}};
use chrono::{Duration, Utc};
use oauth2::{AuthUrl, Client, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, StandardRevocableToken, TokenUrl, basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse}};
use tower_cookies::{Cookie, Cookies};

use crate::{api::{model::{auth::oauth::{OAuthProvider, ProviderUserInfo}, user::User}, repository::auth_repository::AuthRepository, service::{jwt_service::JwtService, user_service::UserService}, state::{ACCESS_TOKEN_COOKIE, ACCESS_TOKEN_TTL_SECS, DEFAULT_USER_TIER, REFRESH_TOKEN_COOKIE, REFRESH_TOKEN_TTL_SECS}}, errors::app_error::AppError};


pub struct OAuthService {
    user_service : Arc<UserService>,
    jwt_service : Arc<JwtService>,
    oauth_repo : AuthRepository,
}

pub type ConfiguredClient = Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet>;

impl OAuthService {

    pub fn new(user_service : Arc<UserService>, jwt_service : Arc<JwtService>, auth_repo : AuthRepository) -> Self {
        OAuthService { 
            user_service: user_service,
            oauth_repo : auth_repo,
            jwt_service : jwt_service
        }
    }

    pub async fn get_or_create_user(
        &self,
        provider: OAuthProvider,
        provider_user_id: String,
        email: String,
        name: String,
        avatar_url: Option<String>,
    ) -> Result<User, AppError> {

        if let Ok(user) = self.user_service.get_user_by_provider(provider, &provider_user_id).await {
            return Ok(user);
        }

        let uuid = self.user_service.create_user(&name, &email, DEFAULT_USER_TIER, provider.clone(), &provider_user_id, avatar_url).await?;
        self.user_service.get_user_by_uuid(&uuid).await
    }

    /*
        Store auth in cache with autodelete
    */
    pub async fn store_oauth_state(&self, state: String, verifier : String) {
        self.oauth_repo.store_auth_verifier(state, verifier);
    }

    /*
        Verify if the value is in cache and delete if it is.
    */
    pub async fn validate_and_consume_oauth_state(&self, state: &str) -> Option<String> {
        self.oauth_repo.verify_and_consume_state(state).await
    }



    // Token issuer

    pub async fn issue_tokens_for_provider(
        &self,
        cookies: &Cookies,
        provider: OAuthProvider,
        info: ProviderUserInfo,
    ) -> Result<Response, AppError> {
        if info.email.is_none() {
            return Err(AppError::auth_error("Authentication provider did not provide email for client."));
        }
        if info.name.is_none() {
            return Err(AppError::auth_error("Authentication provider did not provide name for client."));
        }

        // Upsert user in the store
        let user = self.get_or_create_user(
            provider,
            info.provider_user_id,
            info.email.unwrap(),
            info.name.unwrap(),
            info.avatar_url,
        ).await?;

        tracing::debug!("Authenticated user: {} ({})", user.get_uuid(), user.get_provider());

        // Issue JWT access token (5 min TTL)
        let access_token = self.jwt_service.issue_access_token(&user)?;

        // Issue JWT refresh token (1 day TTL)
        let (refresh_token, jti) =  self.jwt_service.issue_refresh_token(&user)?;

        // Persist the refresh token record for revocation support
        let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS);
         self.jwt_service.store_refresh_token(user.get_uuid().clone(), jti, expires_at).await?;

        // Set HTTP-only cookies
        Self::set_auth_cookies(cookies, &access_token, &refresh_token);

        // Return success response
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Authentication successful",
                "user": {
                    "id": user.get_uuid().to_string(),
                    "email": user.get_email(),
                    "name": user.get_name(),
                    "provider": user.get_provider().to_string(),
                }
            }))
        ).into_response())
    }


    pub fn set_auth_cookies(cookies: &Cookies, access_token: &str, refresh_token: &str) {
        // Access token cookie (5 minutes)
        let access_cookie = Cookie::build((ACCESS_TOKEN_COOKIE, access_token))
            .path("/")
            .http_only(true)
            .secure(true) // Only sent over HTTPS (set to false for local dev)
            .same_site(tower_cookies::cookie::SameSite::Lax)
            .max_age(tower_cookies::cookie::time::Duration::seconds(ACCESS_TOKEN_TTL_SECS))
            .build();

        // Refresh token cookie (1 day)
        let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, refresh_token))
            .path("/")
            .http_only(true)
            .secure(true) // Only sent over HTTPS (set to false for local dev)
            .same_site(tower_cookies::cookie::SameSite::Strict)
            .max_age(tower_cookies::cookie::time::Duration::seconds(REFRESH_TOKEN_TTL_SECS))
            .build();

        cookies.add(access_cookie.into_owned());
        cookies.add(refresh_cookie.into_owned());
    }

    pub fn clear_auth_cookies(cookies: &Cookies) {
        let mut access_cookie = Cookie::build((ACCESS_TOKEN_COOKIE, ""))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(tower_cookies::cookie::SameSite::Lax)
            .build();
        access_cookie.make_removal();

        let mut refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, ""))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(tower_cookies::cookie::SameSite::Lax)
            .build();
        refresh_cookie.make_removal();

        cookies.remove(access_cookie);
        cookies.remove(refresh_cookie); 
    }



    // Client Creation
    
    pub fn google_client(
        &self,
        google_client_id : String,
        google_secret : String,
        google_redirect : String
    ) -> Result<ConfiguredClient, AppError> {
        let basic_client = BasicClient::new(ClientId::new(google_client_id))
        .set_client_secret(ClientSecret::new(google_secret))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).map_err(|_| {
            AppError::auth_error("Failed to parse correct auth uri") // FIXME : add separate app error for this and make it internal server error as status code
        })?)
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).map_err(|_| {
            AppError::auth_error("Failed to parse token uri") // FIXME : same here
        })?)
        .set_redirect_uri(RedirectUrl::new(google_redirect).map_err(|_| {
             AppError::auth_error("Failed to parse redirect uri") // FIXME : same here
        })?);
       
       Ok(basic_client)
    }

    // pub fn github_client(
    //     &self,
    //     config: &Config
    // ) -> Result<ConfiguredClient, AppError> {
    //     let basic_client = BasicClient::new(ClientId::new(config.get_github_client_id().to_string()))
    //     .set_client_secret(ClientSecret::new(config.get_github_client_secret().to_string()))
    //     .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).map_err(|_| {
    //         AppError::auth_error("Failed to parse correct auth uri") // FIXME : add separate app error for this and make it internal server error as status code
    //     })?)
    //     .set_token_uri(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).map_err(|_| {
    //         AppError::auth_error("Failed to parse token uri") // FIXME : same here
    //     })?)
    //     .set_redirect_uri(RedirectUrl::new(config.github_redirect_uri()).map_err(|_| {
    //          AppError::auth_error("Failed to parse redirect uri") // FIXME : same here
    //     })?);
       
    //    Ok(basic_client)
    // }

}