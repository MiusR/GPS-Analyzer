use axum::{extract::{Query, State}, response::{IntoResponse, Redirect}};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse, reqwest};
use serde::Deserialize;

use crate::{api::{model::auth::oauth::{GitHubUserInfo, GoogleUserInfo, OAuthCallback, OAuthProvider, ProviderUserInfo}, state::AppState}, errors::app_error::AppError};



/*
    Api endpoint for oauth with google
*/
pub async fn google_login(
    State(state): State<AppState>
) -> impl IntoResponse {
    let client = match state.get_auth_service().google_client(state.get_config()) {
        Ok(g_client) => g_client,
        Err(err) => {return err.into_response();}
    };
    

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Store the CSRF state token for later verification
    state.store_oauth_state(csrf_token.secret().clone(), pkce_verifier.into_secret());

    tracing::debug!("Redirecting user to Google OAuth2: {}", auth_url);
    Redirect::to(auth_url.as_str()).into_response()
}


/*
    Api endpoint for oauth with google callback
*/
pub async fn google_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AppError> {
    // Surface any error from Google
    if let Some(error) = params.error {
        return Err(AppError::oauth2(&(format!("Google returned error: {error}").to_string())));
    }

    let code = params.code.ok_or(AppError::oauth2("Missing code from google."))?;
    let csrf_state = params.state.ok_or(AppError::oauth2("Google state mismatch."))?;
    let verifier = state.validate_and_consume_oauth_state(&csrf_state).await;
    // Verify CSRF state
    if !verifier.is_none() {
        return Err(AppError::oauth2("Internal state mismatch. Sorry!"));
    }

    let verifier = verifier.unwrap();

    let client = state.get_auth_service().google_client(state.get_config())?;

    let http_client = reqwest::ClientBuilder::new()
    .redirect(reqwest::redirect::Policy::limited(1)) // TODO : might need to increment the amount of redirects
    .build().map_err(|err| {
        AppError::oauth2(&err.to_string())
    })?;

    // Exchange the authorization code for an access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(verifier))
        .request_async(&http_client)
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

        
    // Fetch user info from Google
    let http_client = reqwest::Client::new();
    let response_bytes = http_client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?
        .bytes()
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

    let google_user: GoogleUserInfo = serde_json::from_slice(&response_bytes)
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

    let provider_info = ProviderUserInfo {
        provider_user_id: google_user.id,
        email: google_user.email,
        name: google_user.name,
        avatar_url: google_user.picture,
    };

    state.get_auth_service().issue_tokens_for_provider(&state, OAuthProvider::Google, provider_info).await
}

/*
    Api endpoint for oauth with github
*/
pub async fn github_login(State(state): State<AppState>) -> impl IntoResponse {
      let client = match state.get_auth_service().github_client(state.get_config()) {
        Ok(g_client) => g_client,
        Err(err) => {return err.into_response();}
    };

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    state.store_oauth_state(csrf_token.secret().clone(), csrf_token.into_secret());

    tracing::debug!("Redirecting user to GitHub OAuth2: {}", auth_url);
    Redirect::to(auth_url.as_str()).into_response()
}

/*
    Api endpoint for oauth with github callback
*/
pub async fn github_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallback>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(error) = params.error {
        return Err(AppError::oauth2(&format!("GitHub returned error: {error}")));
    }

    let code = params.code.ok_or(AppError::oauth2("Missing code from google."))?;
    let verifier = state.validate_and_consume_oauth_state(&code).await;
    // Verify CSRF state
    if !verifier.is_none() {
        return Err(AppError::oauth2("Internal state mismatch. Sorry!"));
    }

    let verifier = verifier.unwrap();

    let client = state.get_auth_service().github_client(state.get_config())?;

    let http_client = reqwest::ClientBuilder::new()
    .redirect(reqwest::redirect::Policy::limited(1)) // TODO : might need to increment the amount of redirects
    .build().map_err(|err| {
        AppError::oauth2(&err.to_string())
    })?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(verifier))
        .request_async(&http_client)
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

    // GitHub requires a User-Agent header
   let response_bytes = http_client
        .get("https://api.github.com/user")
        .bearer_auth(token_result.access_token().secret())
        .header("User-Agent", "axum-oauth2-app/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?
        .bytes()
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

    let github_user: GitHubUserInfo = serde_json::from_slice(&response_bytes)
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

    let email = if github_user.email.is_some() {
        github_user.email
    } else {
        fetch_github_primary_email(&http_client, token_result.access_token().secret()).await
    };

    let provider_info = ProviderUserInfo {
        provider_user_id: github_user.id.to_string(),
        email,
        name: github_user.name.or(Some(github_user.login)),
        avatar_url: github_user.avatar_url,
    };

    state.get_auth_service().issue_tokens_for_provider(&state, OAuthProvider::GitHub, provider_info).await
}


// Util for github only :I
#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

async fn fetch_github_primary_email(client: &reqwest::Client, token: &str) -> Option<String> {
    let bytes = client
        .get("https://api.github.com/user/emails")
        .bearer_auth(token)
        .header("User-Agent", "axum-oauth2-app/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .ok()?
        .bytes()
        .await
        .ok()?;

    let emails: Vec<GitHubEmail> = serde_json::from_slice(&bytes).ok()?;

    emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
}
