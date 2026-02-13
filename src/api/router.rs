
use axum::{Router, routing::{get, post}};

use crate::api::{controller::{file_upload::save_request_body, generic::system_info}, state::ServerState};


pub fn build_router(state : ServerState) -> Router {
    Router::new()
    .route("/", get(system_info()))
    .nest("/api", api_router())
    .with_state(state)
}

fn api_router() -> Router<ServerState> {
    Router::new()
    .route("/track/",  post(save_request_body))
}