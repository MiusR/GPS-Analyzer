use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{api::{middleware::auth::AuthenticatedUser, model::{dto::{event_request::{CreateEventRequest, DeleteEventRequest}, tier_request::{CreateTierRequest, GetTierRequest}}, tier::Tier}, service::jwt_service::{JwtService, get_user_uuid_from_claims}, state::AppState}, errors::{app_error::AppError, io_errors::IOError, service_errors::ServiceError}};

/*
    API endpoint for creating a new racing event
*/
pub async fn add_event_for_user(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload) : Json<CreateEventRequest>
) -> impl IntoResponse {

    let user_uuid = get_user_uuid_from_claims(user);

    if let Err(uuid_error) = user_uuid {
        return AppError::service_error(uuid_error).into_response();
    }

    let checked_user_uuid = user_uuid.unwrap();

    if let Err(folder_creation_error) = state.get_event_service().create_event(&checked_user_uuid, &payload.name).await {
        tracing::info!("Failed to create folder :{}", folder_creation_error.to_string());
        return AppError::io_error(IOError::record_operation("event", "Failed to create event space.")).into_response();
    }

    StatusCode::CREATED.into_response()
}

pub async fn delete_event_for_user(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload) : Json<DeleteEventRequest>
) -> impl IntoResponse {
    let user_uuid = get_user_uuid_from_claims(user);

    if let Err(uuid_error) = user_uuid {
        return AppError::service_error(uuid_error).into_response();
    }

    let checked_user_uuid = user_uuid.unwrap();

    if let Err(folder_deletion_error) = state.get_event_service().delete_event(&checked_user_uuid, &payload.name).await {
        tracing::info!("Failed to delete folder :{}", folder_deletion_error.to_string());
        return AppError::io_error(IOError::record_operation("event", "Failed to delete event space.")).into_response();
    }

    StatusCode::OK.into_response()
}



    //TODO : Add a get one/all ? 