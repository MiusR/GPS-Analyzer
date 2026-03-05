use std::sync::Arc;

use uuid::Uuid;

use crate::{api::{model::racing_event::RacingEvent, repository::event_repository::EventRepository, service::file_service::FileService}, errors::service_errors::ServiceError};


#[derive(Clone)]
pub struct EventService {
    file_service : Arc<FileService>,
    event_repository : EventRepository
}


impl EventService {
    pub fn new(event_repository : EventRepository, file_service : Arc<FileService>) -> Self {
        EventService {
            event_repository : event_repository,
            file_service : file_service
        }
    }

    pub async fn create_event(&self, user_uuid : &Uuid, event_name : &str) -> Result<(), ServiceError> {
        self.file_service.create_user_folder(user_uuid, event_name)
        .await
        .map_err(|err| ServiceError::io_error(err))?;

        if let Err(err) = self.event_repository.create_event(event_name, &user_uuid).await {
            let _ = self.file_service.delete_user_folder(user_uuid, event_name)
            .await
            .map_err(|err| ServiceError::io_error(err)); // Why is this not traced?
            return Err(ServiceError::io_error(err));
        }
        
        Ok(())
    }

    pub async fn delete_event(&self, user_uuid : &Uuid, event_name : &str) -> Result<(), ServiceError> {
        self.file_service.delete_user_folder(user_uuid, event_name)
        .await
        .map_err(|err| ServiceError::io_error(err))?;


        if let Err(err) = self.event_repository.delete_event(event_name, &user_uuid).await {
            let _ = self.file_service.create_user_folder(user_uuid, event_name)
            .await
            .map_err(|err| ServiceError::io_error(err)); // Should we even log these
            return Err(ServiceError::io_error(err));
        }

        Ok(())
    }

    pub async fn get_event_by_user_and_name(&self, user_uuid : &Uuid, event_name: &str) -> Result<RacingEvent, ServiceError> {
        self.event_repository.get_event_by_user_and_name(user_uuid, event_name)
        .await
        .map_err(|err| ServiceError::io_error(err))
    }

    pub async fn get_events_by_owner(&self, user_uuid : &Uuid) -> Result<Vec<RacingEvent>, ServiceError> {
        self.event_repository.get_events_by_owner(user_uuid)
        .await
        .map_err(|err| {ServiceError::io_error(err)})
    }
}