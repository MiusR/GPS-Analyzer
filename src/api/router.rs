
use axum::{Router, routing::{get, post}};

use crate::api::{controller::{ file_controller::{download_from_temp, save_to_temp}, generic::system_info, tier_controller::{add_tier, get_tier_info}, user_controller::{create_user, delete_user, get_user, update_user}}, state::ServerState};

/*
    Creates the main app router using the @state
*/
pub fn build_router(state : ServerState) -> Router {
    Router::new()
    .route("/", get(system_info()))
    .nest("/api", api_router())
    .with_state(state)
}

/*
    Creates the router holding only the track & analysis related endpoints
*/
fn api_router() -> Router<ServerState> {
    Router::new()
    .route("/track/",  post(save_to_temp).get(download_from_temp))
    .route("/tier/", get(get_tier_info).post(add_tier))
    .route("/user/", get(get_user).post(create_user).put(update_user).delete(delete_user))
}