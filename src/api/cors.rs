use std::time::Duration;

use axum::http::{HeaderValue, Method, header};
use tower_http::cors::CorsLayer;


/*
    Creates a new CorsLayer that allows the origin localhost:@local_port to do @methods
*/
pub fn build_cors_layer(local_port : &str, methods : Vec<Method>) -> CorsLayer {
    CorsLayer::new()
    .allow_credentials(true)
    .allow_origin(("http://localhost:".to_string() +  local_port).to_owned().parse::<HeaderValue>().expect("Failed to set allowed origin"))
    .allow_methods(methods)
    .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    .max_age(Duration::from_secs(3600))
}