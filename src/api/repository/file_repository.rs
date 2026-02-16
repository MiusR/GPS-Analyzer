use axum::{
    BoxError, body::Bytes
};

use futures_util::{Stream, TryStreamExt};
use std::{io, pin::pin};
use tokio::{fs::File, io::{AsyncWriteExt, BufWriter}};
use tokio_util::io::{ReaderStream, StreamReader};

use crate::errors::io_errors::IOError;

#[derive(Clone)]
pub struct FileRepository;


impl FileRepository {
    pub fn new() -> Self {
        FileRepository {}
    }

    /*
    Returns a pair of (HeaderName, Body) file with given @origin_path
    */
    pub async fn stream_from_file(
        &self,
        origin_path : &str
    ) -> Result<ReaderStream<File>, IOError> {
        let file = File::open(origin_path)
            .await
            .map_err(|err| {
                tracing::warn!("Failed to open file: {}", err);
                return IOError::invalid_path("file stream", &("Failed to open file with provided name ".to_string() + origin_path))
            })?;
        let reader = ReaderStream::new(file);
        Ok(reader)
    }

    /*
        Creates file with given @file_name in the @secured_path with its content read from the @stream
        This function expects the @secured_path to be correct and not mallicous!
    */
    pub async fn stream_to_file<S, E>(
        &self,
        file_name : &str,
        secured_path : &str,
        stream : S
    ) -> Result<(), IOError>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        if !Self::path_is_valid(file_name) {
            tracing::error!("Upload file request contains illegal arguments in path {}", file_name);
            return Err(IOError::invalid_path("file stream", "File path contains invalid arguments!"));
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
            return IOError::stream_error("file stream", "Uh oh! Failed to save file from stream, please try again!");
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

}
