use axum::Json;
use chrono::{Duration, Utc};
use oauth2::{AuthUrl, Client, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, StandardRevocableToken, TokenUrl, basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse, BasicTokenType}};

use crate::{api::{model::{auth::oauth::{OAuthProvider, ProviderUserInfo}, config::Config, dto::jwt_request::TokenResponse}, service::jwt_service::{ACCESS_TOKEN_TTL_SECS, REFRESH_TOKEN_TTL_SECS}, state::AppState}, errors::app_error::AppError};



pub struct OAuthService();

type ConfiguredClient = Client<
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
    // Token issuer
    pub async fn issue_tokens_for_provider(
        &self,
        state: &AppState,
        provider: OAuthProvider,
        info: ProviderUserInfo,
    ) -> Result<Json<TokenResponse>, AppError> {
        // Upsert user in the store
        let user = state.upsert_user(
            provider,
            info.provider_user_id,
            info.email,
            info.name,
            info.avatar_url,
        );

        tracing::debug!("Authenticated user: {} ({})", user.get_uuid(), user.get_provider());

        // Issue JWT access token (5 min TTL)
        let access_token = state.get_jwt_service().issue_access_token(&user)?;

        // Issue JWT refresh token (1 day TTL)
        let (refresh_token, jti) = state.get_jwt_service().issue_refresh_token(&user)?;

        // Persist the refresh token record for revocation support
        let expires_at = Utc::now() + Duration::seconds(REFRESH_TOKEN_TTL_SECS);
        state.store_refresh_token(user.get_uuid(), jti, expires_at);

        Ok(Json(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: ACCESS_TOKEN_TTL_SECS as u64,
        }))
    }

    // Client Creation
    
    pub fn google_client(
        &self,
        config: &Config
    ) -> Result<ConfiguredClient, AppError> {
        let basic_client = BasicClient::new(ClientId::new(config.get_google_client_id().to_string()))
        .set_client_secret(ClientSecret::new(config.get_google_client_secret().to_string()))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).map_err(|_| {
            AppError::auth_error("Failed to parse correct auth uri") // FIXME : add separate app error for this and make it internal server error as status code
        })?)
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).map_err(|_| {
            AppError::auth_error("Failed to parse token uri") // FIXME : same here
        })?)
        .set_redirect_uri(RedirectUrl::new(config.google_redirect_uri()).map_err(|_| {
             AppError::auth_error("Failed to parse redirect uri") // FIXME : same here
        })?);
       
       Ok(basic_client)
    }

    pub fn github_client(
        &self,
        config: &Config
    ) -> Result<ConfiguredClient, AppError> {
        let basic_client = BasicClient::new(ClientId::new(config.get_github_client_id().to_string()))
        .set_client_secret(ClientSecret::new(config.get_github_client_secret().to_string()))
        .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).map_err(|_| {
            AppError::auth_error("Failed to parse correct auth uri") // FIXME : add separate app error for this and make it internal server error as status code
        })?)
        .set_token_uri(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).map_err(|_| {
            AppError::auth_error("Failed to parse token uri") // FIXME : same here
        })?)
        .set_redirect_uri(RedirectUrl::new(config.github_redirect_uri()).map_err(|_| {
             AppError::auth_error("Failed to parse redirect uri") // FIXME : same here
        })?);
       
       Ok(basic_client)
    }

}