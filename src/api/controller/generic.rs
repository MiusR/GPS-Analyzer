// Api endpoint
pub fn system_info() -> &'static str {
    "Hello world! \n Running GPS-Analyzer v0.2.0"
}

// GET /health
pub async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
