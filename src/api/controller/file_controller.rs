
use axum::{Json, body::Body, extract::State, http::{Response, StatusCode, header}, response::IntoResponse
};

use crate::api::{middleware::auth::AuthenticatedUser, model::dto::file_request::{DownloadRequest, UploadCompleted}, state::AppState};


/*
    API endpoint for saving request body to file on server
*/
pub async fn save_to_temp(
    AuthenticatedUser(_): AuthenticatedUser,
    State(state) : State<AppState>,
    request : Body
) -> impl IntoResponse {
    let mut stream =request.into_data_stream(); 
    let result = state.get_file_service().save_to_temp(&mut stream)
    .await
    .map_err( |err| {return err.into_response();});
    if let Err(response) = result { 
        return response;
    }else {
        (StatusCode::CREATED, axum::Json::from(UploadCompleted{ file_name : result.unwrap()})).into_response()
    }
}

/*
    API endpoint for downloading file from server
*/
pub async fn download_from_temp(
    AuthenticatedUser(_): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload): Json<DownloadRequest>,
) -> impl IntoResponse {
    match state.get_file_service().download_from_temp(&payload.path).await {
        Ok(stream) => {
            Response::builder()
            .header(header::CONTENT_TYPE, "application/gpx; charset=utf-8")
            .header(header::CONTENT_DISPOSITION, "attachment; filename=\"track.txt\"") // TODO : if track is saved give back actual name
            .status(StatusCode::OK)
            .body(Body::from_stream(stream)).map_err(|err| {
                tracing::warn!("Could not build body from stream: {}", err.to_string());
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            } )
            .into_response()
        },
        Err(err) => err.into_response()
    }
}