// TODO make user endpoints

use axum::{Json, extract::State, response::IntoResponse};

use crate::{api::{model::dto::user_requests::{CreateUserRequest, DeleteUserRequest, GetUserRequest, UpdateUserRequest}, state::ServerState}, errors::{app_error::AppError, service_errors::ServiceError}};


pub async fn get_user(
    State(state) : State<ServerState>,
    Json(payload) : Json<GetUserRequest>
) -> impl IntoResponse {
    if let Some(email) = payload.email {
        match state.get_user_service().get_user_by_email(&email).await {
            Ok(user) => {return axum::Json::from(user).into_response()},
            Err(err) => {return err.into_response();}
            
        }
    }

    if let Some(uuid) = payload.uuid {
        match state.get_user_service().get_user_by_uuid(&uuid).await {
            Ok(user) => {return axum::Json::from(user).into_response()},
            Err(err) => {return err.into_response();}
            
        }
    }

    AppError::service_error(ServiceError::invalid_data("Request must contain either a valid user uuid or email.")).into_response()
}


pub async fn create_user(
    State(state) : State<ServerState>,
    Json(payload) : Json<CreateUserRequest>
) -> impl IntoResponse {
    match state.get_user_service().create_user(&payload.name, &payload.email, &payload.tier).await {
        Ok(uuid) => {return axum::Json::from(uuid).into_response();},
        Err(err) => {return err.into_response();}
    }
}

pub async fn update_user(
    State(state) : State<ServerState>,
    Json(payload) : Json<UpdateUserRequest>
) -> impl IntoResponse {
    match state.get_user_service().update_user(&payload.uuid, payload.name, payload.email, payload.tier).await {
        Ok(user) => {return axum::Json::from(user).into_response();},
        Err(err) => {return err.into_response();}
    }
}

pub async fn delete_user(
    State(state) : State<ServerState>,
    Json(payload) : Json<DeleteUserRequest>
) -> impl IntoResponse {
    match state.get_user_service().delete_user(&payload.uuid).await {
        Ok(user) => {return axum::Json::from(user).into_response();},
        Err(err) => {return err.into_response();}
    }
}

