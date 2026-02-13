use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;



pub fn build_cors_layer(local_port : &str, methods : Vec<Method>) -> CorsLayer {
    CorsLayer::new()
    .allow_origin(("http://localhost:".to_string() +  local_port).to_owned().parse::<HeaderValue>().expect("Failed to set allowed origin"))
    .allow_methods(methods)
}