use axum::body::BodyDataStream;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{api::repository::file_repository::FileRepository, errors::{app_error::AppError, io_errors::IOError}};



#[derive(Clone)]
pub struct FileService {
    file_repo : FileRepository
}

impl FileService {
    
    #[allow(dead_code)]
    const UPLOADS_DIRECTORY: &str = "uploads";
    const UPLOADS_TEMP_DIRECTORY : &str = "uploads/temp";
    const UPLOADS_USERS_DIRECTORY : &str = "uploads/users";

    pub fn new() -> Self {
        FileService { file_repo : FileRepository::new()}
    }

    /*
        Init resources required for track file upload via api
    */
    pub async fn init() {
        tokio::fs::create_dir_all("./".to_string() + Self::UPLOADS_TEMP_DIRECTORY)
        .await
        .expect("Failed to create temp uploads directory");
        tokio::fs::create_dir_all("./".to_string() + Self::UPLOADS_USERS_DIRECTORY)
        .await
        .expect("Failed to create users uploads directory");
    }


    pub async fn save_to_temp(&self, stream : &mut BodyDataStream) -> Result<String, AppError> {
        let temp_file_name = format!("{}.gpx", Uuid::new_v4());

        if let Err(error) = self.file_repo.stream_to_file(
            &temp_file_name, 
            Self::UPLOADS_TEMP_DIRECTORY, 
            stream
        ).await {
            return Err(AppError::io_error(error));
        }

        Ok(temp_file_name)
    }

    pub async fn download_from_temp(&self, path : &str) -> Result<ReaderStream<File>, AppError> {
        if !FileRepository::path_is_valid(&path) {
            tracing::error!("Download file request contains illegal arguments in file name {}", &path);
            return Err(AppError::io_error(IOError::invalid_path("downloads", "Invalid path name!")));
        }

        let origin_path = std::path::Path::new(Self::UPLOADS_TEMP_DIRECTORY).join(format!("{}.gpx", &path));

        match self.file_repo.stream_from_file(origin_path.to_str().unwrap_or("none.txt")).await {
            Ok (res) => Ok(res),
            Err(err) => Err(AppError::io_error(err))
        }
    }
}