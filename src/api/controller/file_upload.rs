

use axum::{
    body::Bytes,
    extract::{Path, Request},
    http::StatusCode,
    BoxError,
};
use futures_util::{Stream, TryStreamExt};
use uuid::Uuid;
use std::{io, pin::pin};
use tokio::{fs::File, io::{AsyncWriteExt, BufWriter}};
use tokio_util::io::StreamReader;

const UPLOADS_DIRECTORY: &str = "uploads";

// TODO : make confing file from which to read path to uploads

/*
    Init resources required for track file upload via api
 */
pub async fn init() {
    tokio::fs::create_dir_all("./".to_string() + UPLOADS_DIRECTORY + "/temp")
    .await
    .expect("Failed to create temp uploads directory");
    tokio::fs::create_dir_all("./".to_string() + UPLOADS_DIRECTORY + "/users")
    .await
    .expect("Failed to create users uploads directory");
}

/*
    API endpoint for saving request body to file on server
*/
pub async fn save_request_body(
    request : Request
) -> Result<(), (StatusCode, String)> {
    stream_to_file(&(Uuid::new_v4().to_string() + ".gpx"), "temp", request.into_body().into_data_stream()).await
}


// FIXME : file names should be given by a uuid and we need to update the internal file loader so that the data about the tracks arent loaded from file name
/*
    Creates file with given path
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
        tracing::error!("Provided file from request contains illegal arguments in path {}", file_name);
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

   async {
        let body_with_io_error = stream.map_err(io::Error::other);
        let mut body_reader = pin!(StreamReader::new(body_with_io_error));

        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(secured_path.trim_start_matches("/")).join(file_name.trim_start_matches("/"));
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
    Checks if a path is valid and does not contain any illegal characters
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