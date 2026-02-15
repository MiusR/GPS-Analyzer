use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DownloadRequest {
    pub path: String,
}

#[derive(Serialize)]
pub struct UploadCompleted {
    pub file_name : String,
}