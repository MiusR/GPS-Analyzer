use axum::{
    BoxError, body::{Body, Bytes}, http::{HeaderName, StatusCode, header}
};

use futures_util::{Stream, TryStreamExt};
use std::{io, pin::pin};
use tokio::{fs::File, io::{AsyncWriteExt, BufWriter}};
use tokio_util::io::{ReaderStream, StreamReader};


/*
    Returns a pair of (HeaderName, Body) file with given @origin_path
*/
pub async fn stream_from_file(
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
        (header::CONTENT_TYPE, "text/gpx; charset=utf-8"),
        (header::CONTENT_DISPOSITION, "attachment; filename=\"track.txt\""), // TODO : if track is saved give back actual name
    ];
    Ok((headers, body))
}

/*
    Creates file with given @file_name in the @secured_path with its content read from the @stream
    This function expects the @secured_path to be correct and not mallicous!
*/
pub async fn stream_to_file<S, E>(
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
pub fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }
    components.count() == 1
}