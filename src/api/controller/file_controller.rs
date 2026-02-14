use axum::{
    BoxError, Json, body::{Body, Bytes}, extract::{Path, Request}, http::{HeaderName, StatusCode, header}, response::IntoResponse
};

use futures_util::{Stream, TryStreamExt};
use uuid::Uuid;
use std::{io, pin::pin};
use tokio::{fs::File, io::{AsyncWriteExt, BufWriter}};
use tokio_util::io::{ReaderStream, StreamReader};

use crate::api::dto::file_request::DownloadRequest;

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





// ================== UTIL FUNCTIONS ==================

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


async fn stream_from_file(
    origin_path : &str
) -> Result<([(HeaderName, &str); 2], Body), StatusCode> {
    let file = File::open(origin_path)
        .await
        .map_err(|err| {
            tracing::warn!("Failed to open file: {}", err);
            StatusCode::NOT_FOUND
        })?;
    let reader = ReaderStream::new(file);
    let body = Body::from_stream(reader);
    let headers = [
        (header::   CONTENT_TYPE, "text/gpx; charset=utf-8"),
        (header::CONTENT_DISPOSITION, "attachment; filename=\"track.txt\""), // TODO : if track is saved give back actual name
    ];
    Ok((headers, body))
}

/*
    Creates file with given @file_name in the @secured_path with its content read from the @stream
    This function expects the @secured_path to be correct and not mallicous!
*/
async fn stream_to_file<S, E>(
    file_name : &str,
    secured_path : &str,
    stream : S
) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(file_name) {
        tracing::error!("Upload file request contains illegal arguments in path {}", file_name);
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

   async {
        let body_with_io_error = stream.map_err(io::Error::other);
        let mut body_reader = pin!(StreamReader::new(body_with_io_error));

        let path = std::path::Path::new(secured_path.trim_start_matches("/")).join(file_name.trim_start_matches("/"));
        let mut file = BufWriter::new(File::create(&path).await?);

        let bytes_written = tokio::io::copy(&mut body_reader, &mut file).await?;
        
        file.flush().await?;
        file.get_mut().shutdown().await?;

        tracing::info!("Successfully wrote {} bytes to {:?}", bytes_written, path);
        
        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| {
        tracing::error!("Failed to save file from stream: {}", err.to_string());
        return (StatusCode::INTERNAL_SERVER_ERROR, "Uh oh! Something went wrong, please try again after a bit!".to_string());
    })
}


/*
    Checks if a path is valid and does not contain any illegal characters that are not considered path::Component::Normal
*/
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }
    components.count() == 1
}