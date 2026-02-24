use axum::{extract::{Query, State}, response::{IntoResponse, Redirect}};
use tower_cookies::Cookies;

use crate::{api::{model::auth::oauth::{OAuthCallback, OAuthProvider}, state::AppState, util::auth_util::{build_google_claims_request, fetch_provider_info, fetch_token_response}}, errors::app_error::AppError};



/*
    Api endpoint for oauth with google
*/
pub async fn google_login(
    State(state): State<AppState>
) -> impl IntoResponse {
    let config = state.get_config();
    let client = match state.get_auth_service().google_client(config.get_google_client_id().to_string(), config.get_google_client_secret().to_string(), config.google_redirect_uri().to_string()) {
        Ok(g_client) => g_client,
        Err(err) => {return err.into_response();}
    };
    
    let (auth_url, csrf_token, pkce_verifier) = build_google_claims_request(&client);
    
    state.get_auth_service().store_oauth_state(csrf_token.secret().clone(), pkce_verifier.into_secret()).await;

    Redirect::to(auth_url.as_str()).into_response()
}


/*
    Api endpoint for oauth with google callback
*/
pub async fn google_callback(
    Query(params): Query<OAuthCallback>,
    State(state): State<AppState>,
    cookie : Cookies,
) -> Result<impl IntoResponse, AppError> {
    // Surface any error from Google
    if let Some(error) = params.error {
        tracing::warn!("Google returned error: {error}");
        return Err(AppError::oauth2("OAuth failed, please try again!"));
    }

    let code = params.code.ok_or(AppError::oauth2("Missing code from google."))?;
    let csrf_state = params.state.ok_or(AppError::oauth2("Google state mismatch."))?;

    let verifier = state.get_auth_service()
    .validate_and_consume_oauth_state(&csrf_state)
    .await
    .ok_or(AppError::oauth2("Internal state mismatch. Sorry!"))?;

    let config = state.get_config();
    let client = state.get_auth_service().google_client(config.get_google_client_id().to_string(), config.get_google_client_secret().to_string(), config.google_redirect_uri().to_string())?;

    let token_result = fetch_token_response(client, code, verifier).await?;

    let google_info_url = "https://www.googleapis.com/oauth2/v2/userinfo";
    let provider_info = fetch_provider_info(google_info_url, &token_result).await?;

    state.get_auth_service().issue_tokens_for_provider(&cookie, OAuthProvider::Google, provider_info).await?;
    Ok(Redirect::to("http://localhost:5173/dashboard").into_response()) // TODO : this should not be a magic variable and it should be an env var
}

// /*
//     Api endpoint for oauth with github
// */
// pub async fn github_login(State(state): State<AppState>) -> impl IntoResponse {
//       let client = match state.get_auth_service().github_client(state.get_config()) {
//         Ok(g_client) => g_client,
//         Err(err) => {return err.into_response();}
//     };

//     let (auth_url, csrf_token) = client
//         .authorize_url(CsrfToken::new_random)
//         .add_scope(Scope::new("read:user".to_string()))
//         .add_scope(Scope::new("user:email".to_string()))
//         .url();

//     state.get_auth_service().store_oauth_state(csrf_token.secret().clone(), csrf_token.into_secret()).await;

//     tracing::debug!("Redirecting user to GitHub OAuth2: {}", auth_url);
//     Redirect::to(auth_url.as_str()).into_response()
// }

// /*
//     Api endpoint for oauth with github callback
// */
// pub async fn github_callback(
//     Query(params): Query<OAuthCallback>,
//     State(state): State<AppState>,
//     cookie : Cookies,
// ) -> Result<Response, AppError> {
//     if let Some(error) = params.error {
//         return Err(AppError::oauth2(&format!("GitHub returned error: {error}")));
//     }

//     let code = params.code.ok_or(AppError::oauth2("Missing code from google."))?;
//     let verifier = state.get_auth_service().validate_and_consume_oauth_state(&code).await;
//     // Verify CSRF state
//     if verifier.is_none() {
//         return Err(AppError::oauth2("Internal state mismatch. Sorry!"));
//     }

//     let verifier = verifier.unwrap();

//     let client = state.get_auth_service().github_client(state.get_config())?;

//     let http_client = reqwest::ClientBuilder::new()
//     .redirect(reqwest::redirect::Policy::limited(1)) // TODO : might need to increment the amount of redirects
//     .build().map_err(|err| {
//         AppError::oauth2(&err.to_string())
//     })?;

//     let token_result = client
//         .exchange_code(AuthorizationCode::new(code))
//         .set_pkce_verifier(PkceCodeVerifier::new(verifier))
//         .request_async(&http_client)
//         .await
//         .map_err(|e| AppError::oauth2(&e.to_string()))?;

//     // GitHub requires a User-Agent header
//    let response_bytes = http_client
//         .get("https://api.github.com/user")
//         .bearer_auth(token_result.access_token().secret())
//         .header("User-Agent", "axum-oauth2-app/1.0")
//         .header("Accept", "application/vnd.github.v3+json")
//         .send()
//         .await
//         .map_err(|e| AppError::oauth2(&e.to_string()))?
//         .bytes()
//         .await
//         .map_err(|e| AppError::oauth2(&e.to_string()))?;

//     let github_user: GitHubUserInfo = serde_json::from_slice(&response_bytes)
//         .map_err(|e| AppError::oauth2(&e.to_string()))?;

//     let email = if github_user.email.is_some() {
//         github_user.email
//     } else {
//         fetch_github_primary_email(&http_client, token_result.access_token().secret()).await
//     };

//     let provider_info = ProviderUserInfo {
//         provider_user_id: github_user.id.to_string(),
//         email,
//         name: github_user.name.or(Some(github_user.login)),
//         avatar_url: github_user.avatar_url,
//     };

//     state.get_auth_service().issue_tokens_for_provider(&state, &cookie, OAuthProvider::GitHub, provider_info).await
// }