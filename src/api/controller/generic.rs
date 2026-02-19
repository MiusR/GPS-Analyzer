// Api endpoint
pub fn landing() -> &'static str {
    "Hello world! "
}

// GET /health
pub async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
