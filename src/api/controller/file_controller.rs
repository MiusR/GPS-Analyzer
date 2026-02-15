use axum::{Json, extract::Request, http::{StatusCode}, response::IntoResponse
};
use uuid::Uuid;

use crate::api::{io::file_repo::{path_is_valid, stream_from_file, stream_to_file}, model::dto::file_request::DownloadRequest};

#[allow(dead_code)]
const UPLOADS_DIRECTORY: &str = "uploads";
const UPLOADS_TEMP_DIRECTORY : &str = "uploads/temp";
const UPLOADS_USERS_DIRECTORY : &str = "uploads/users";

// ================== END POINTS ==================

/*
    API endpoint for saving request body to file on server
*/
pub async fn save_to_temp(
    request : Request
) -> Result<(), (StatusCode, String)> {
    stream_to_file(&(Uuid::new_v4().to_string() + ".gpx"), UPLOADS_TEMP_DIRECTORY, request.into_body().into_data_stream()).await
}

/*
    API endpoint for downloading file from server
*/
pub async fn download_from_temp(
    Json(payload): Json<DownloadRequest>,
) -> impl IntoResponse {
    if !path_is_valid(&payload.path) {
        tracing::error!("Download file request contains illegal arguments in file name {}", &payload.path);
        return StatusCode::BAD_REQUEST.into_response();
    }

    let origin_path = std::path::Path::new(UPLOADS_TEMP_DIRECTORY).join(&payload.path).join(".gpx");

    stream_from_file(origin_path.to_str().unwrap_or("none.txt")).await.into_response()
}



// TODO : Maybe move this to file_repo since it is related more to file cration than api endpoints, find a way to pass the directories downards
// UTIL 
/*
    Init resources required for track file upload via api
*/
pub async fn init() {
    tokio::fs::create_dir_all("./".to_string() + UPLOADS_TEMP_DIRECTORY)
    .await
    .expect("Failed to create temp uploads directory");
    tokio::fs::create_dir_all("./".to_string() + UPLOADS_USERS_DIRECTORY)
    .await
    .expect("Failed to create users uploads directory");
}