use std::sync::Arc;

use uuid::Uuid;

use crate::{api::{repository::event_repository::{self, EventRepository}, service::file_service::FileService}, errors::service_errors::ServiceError};


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
            self.file_service.delete_user_folder(user_uuid, event_name)
            .await
            .map_err(|err| ServiceError::io_error(err))?;
            return Err(ServiceError::io_error(err));
        }
        
        Ok(())
    }
}