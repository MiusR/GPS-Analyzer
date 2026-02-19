use oauth2::{AuthorizationCode, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier, Scope, StandardTokenResponse, TokenResponse, basic::BasicTokenType, reqwest, url::Url};

use crate::{api::{model::auth::oauth::{GoogleUserInfo, ProviderUserInfo}, service::oauth_service::ConfiguredClient}, errors::app_error::AppError};


/*
    Builds a request that can be sent to google auth servers for authentication
    Returns (authentication url, csrf token, pkce code verifier)
*/
pub fn build_google_claims_request(client : &ConfiguredClient) -> (Url, CsrfToken, PkceCodeVerifier) {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();
    (auth_url, csrf_token, pkce_verifier)

}

pub async fn fetch_token_response(client : ConfiguredClient, code :String, verifier : String) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, AppError>{
    let http_client = reqwest::ClientBuilder::new()
    .redirect(reqwest::redirect::Policy::limited(1))
    .build().map_err(|err| {
        tracing::warn!("Failed to build http client for google auth : {}", err.to_string());
        AppError::oauth2("OAuth failed to build client.")
    })?;

    // Exchange the authorization code for an access token
    client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(verifier))
        .request_async(&http_client)
        .await
        .map_err(|e| {
            tracing::warn!("Failed to get request token result : {}", &e.to_string());
            AppError::oauth2("OAuth failed to retrieve token")
    })
}

pub async fn fetch_provider_info(website : &str, token_result : &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>) -> Result<ProviderUserInfo, AppError> {
    let http_client = reqwest::Client::new();
    let response_bytes = http_client
        .get(website)
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?
        .bytes()
        .await
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

    let google_user: GoogleUserInfo = serde_json::from_slice(&response_bytes)
        .map_err(|e| AppError::oauth2(&e.to_string()))?;

    Ok(ProviderUserInfo {
        provider_user_id: google_user.id,
        email: google_user.email,
        name: google_user.name,
        avatar_url: google_user.picture,
    })
}



// // Util for github only :I
// #[derive(Deserialize)]
// struct GitHubEmail {
//     email: String,
//     primary: bool,
//     verified: bool,
// }

// async fn fetch_github_primary_email(client: &reqwest::Client, token: &str) -> Option<String> {
//     let bytes = client
//         .get("https://api.github.com/user/emails")
//         .bearer_auth(token)
//         .header("User-Agent", "axum-oauth2-app/1.0")
//         .header("Accept", "application/vnd.github.v3+json")
//         .send()
//         .await
//         .ok()?
//         .bytes()
//         .await
//         .ok()?;

//     let emails: Vec<GitHubEmail> = serde_json::from_slice(&bytes).ok()?;

//     emails
//         .into_iter()
//         .find(|e| e.primary && e.verified)
//         .map(|e| e.email)
// }
