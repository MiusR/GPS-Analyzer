
use axum::{Json, extract::Request, http::{StatusCode}, response::IntoResponse
};
use uuid::Uuid;

use crate::{api::{io::file_repo::{path_is_valid, stream_from_file, stream_to_file}, model::dto::file_request::{DownloadRequest, UploadCompleted}}, errors::{app_error::AppError, io_errors::IOError}};

#[allow(dead_code)]
const UPLOADS_DIRECTORY: &str = "uploads";
const UPLOADS_TEMP_DIRECTORY : &str = "uploads/temp";
const UPLOADS_USERS_DIRECTORY : &str = "uploads/users";



/*
    API endpoint for saving request body to file on server
*/
pub async fn save_to_temp(
    request : Request
) -> impl IntoResponse {
    let temp_file_name = format!("{}.gpx", Uuid::new_v4());

    if let Err(error) = stream_to_file(
        &temp_file_name, 
        UPLOADS_TEMP_DIRECTORY, 
        request.into_body().into_data_stream()
    ).await {
        return AppError::io_error(error).into_response();
    }

    (StatusCode::CREATED, axum::Json::from(UploadCompleted{ file_name : temp_file_name})).into_response()
}

/*
    API endpoint for downloading file from server
*/
pub async fn download_from_temp(
    Json(payload): Json<DownloadRequest>,
) -> impl IntoResponse {
    if !path_is_valid(&payload.path) {
        tracing::error!("Download file request contains illegal arguments in file name {}", &payload.path);
        return AppError::io_error(IOError::invalid_path("downloads", "Invalid path name!")).into_response();
    }

    let origin_path = std::path::Path::new(UPLOADS_TEMP_DIRECTORY).join(&payload.path).join(".gpx");

    match stream_from_file(origin_path.to_str().unwrap_or("none.txt")).await {
        Ok (res) => res.into_response(),
        Err(err) => AppError::io_error(err).into_response()
    }
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