
use axum::{Router, routing::{get, post}};
use tower_cookies::CookieManagerLayer;
use tower_http::limit::RequestBodyLimitLayer;
use crate::api::{controller::{ auth_controller::{google_callback, google_login}, file_controller::{download_from_temp, save_to_temp}, generic::{health, landing}, tier_controller::{add_tier, get_tier_info}, token_controller::{logout_all, refresh_token, revoke_token}, user_controller::{delete_user, get_me, get_user, update_user}}, state::AppState};

const FILE_SIZE_LIMIT : usize = 1024;

/*
    Creates the main app router using the @state
*/
pub fn build_router(state : AppState) -> Router {
    Router::new()
    .route("/", get(landing()))
    .route("/health", get(health))
    .nest("/api", api_router())
    .nest("/auth", auth_router())
    .layer(CookieManagerLayer::new())
    .layer(RequestBodyLimitLayer::new(FILE_SIZE_LIMIT * FILE_SIZE_LIMIT))// 1MB
    .with_state(state)
}

/*
    Creates the router holding only the track & analysis related endpoints
*/
fn api_router() -> Router<AppState> {
    Router::new()
    .route("/track/",  post(save_to_temp).get(download_from_temp))
    .route("/tier/", get(get_tier_info).post(add_tier))
    .route("/user/", get(get_user).put(update_user).delete(delete_user))
}

fn auth_router() -> Router<AppState> {
    Router::new()
    //  Step 1 - Login
    .route("/google", get(google_login))
    //  .route("/github", get(github_login))
    //  Step 2 - provider redirects back here with a code
    .route("/google/callback", get(google_callback))
    //.route("/github/callback", get(github_callback))
    // Token management
    .route("/refresh", post(refresh_token))
    .route("/revoke", post(revoke_token))
    .route("/logout-all", post(logout_all))
    
    //Util - atuenticated
    .route("/me", get(get_me))
}