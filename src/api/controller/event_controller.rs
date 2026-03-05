
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use crate::{api::{middleware::auth::AuthenticatedUser, model::dto::event_request::{CreateEventRequest, DeleteEventRequest, GetEventsRequest}, service::jwt_service::get_user_uuid_from_claims, state::AppState}, errors::{app_error::AppError, io_errors::IOError}};


// TODO : All endpoints should follow new standard of return Result<impl IntoResponse, AppError>
// TODO : Look into proper logging of information to avoid attack vectors 
/*
    API endpoint for creating a new racing event
*/
pub async fn add_event_for_user(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload) : Json<CreateEventRequest>
) -> Result<impl IntoResponse, AppError> {

    let user_uuid = get_user_uuid_from_claims(user)
    .map_err(|err| {
        AppError::service_error(err)
    })?;

    state.get_event_service().create_event(&user_uuid, &payload.name)
    .await
    .map_err(|err| {
        tracing::info!("Failed to create folder :{}", err.to_string());
        return AppError::io_error(IOError::record_operation("event", "Failed to create event space."));
    })?;

    Ok(StatusCode::CREATED)
}

/*
    API endpoint for deleting a user event
*/
pub async fn delete_event_for_user(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload) : Json<DeleteEventRequest>
) -> Result<impl IntoResponse, AppError> {
    let user_uuid = get_user_uuid_from_claims(user)
    .map_err(|err| {
        AppError::service_error(err)
    })?;

    state.get_event_service().delete_event(&user_uuid, &payload.name)
    .await
    .map_err(|err| {
        tracing::info!("Failed to delete folder :{}", err.to_string());
        return AppError::io_error(IOError::record_operation("event", "Failed to delete event space."))
    })?;

    Ok(StatusCode::OK)
}


/*
    API endpoint for retrieveing all events owned by a user or a single event 
*/
pub async fn get_events_for_user(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state) : State<AppState>,
    Json(payload) : Json<GetEventsRequest>
) -> Result<impl IntoResponse, AppError> {
    let user_uuid = get_user_uuid_from_claims(user)
    .map_err(|err| {
        AppError::service_error(err)
    })?;

    match payload.name {
        Some(value) => {
            let event = state.get_event_service().get_event_by_user_and_name(&user_uuid, &value)
            .await
            .map_err(|err| {
                return AppError::service_error(err);
            })?;

            return Ok(Json(vec![event]));
        
        },
        None => {
                let events = state.get_event_service().get_events_by_owner(&user_uuid)
                .await
                .map_err(|err| {
                    return AppError::service_error(err);
                })?;

                return Ok(Json(events));
        }
    }
}
