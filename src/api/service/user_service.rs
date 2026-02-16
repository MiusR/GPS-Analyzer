use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{api::{model::user::{User, UserEmail}, repository::user_repository::UserRepository, service::tier_service::TierService}, errors::app_error::AppError};




#[derive(Clone)]
pub struct UserService {
    user_repository : UserRepository,
    tier_service : Arc<TierService>
}


impl UserService {
    pub fn new(pg_pool : PgPool, tier_service : Arc<TierService>) -> Self {
        UserService { user_repository: UserRepository::new(pg_pool), tier_service: tier_service }
    }

    pub async fn delete_user(&self, uuid : &Uuid) -> Result<User, AppError> {
        self.user_repository.delete_user(uuid).await.map_err(|err| {
            AppError::io_error(err)
        })
    }

    pub async fn create_user(&self, name : &str, email : &str, tier_name : &str) -> Result<Uuid, AppError> {
        let tier = self.tier_service.get_tier_by_name(tier_name).await?;
        self.user_repository.add_user(name, email, tier).await.map_err(|err| {
            AppError::io_error(err)
        })
    }

    pub async fn get_user_by_uuid(&self, uuid: &Uuid) -> Result<User, AppError> {
        let partial_user  = self.user_repository.get_user_by_uuid(uuid).await.map_err(|err| {
            AppError::io_error(err)
        })?;
        let tier = self.tier_service.get_tier_by_uuid(&partial_user.get_tier().uuid).await?;
        let user = User::new(
            &partial_user.get_uuid().to_string(), 
            &partial_user.get_name().to_string(), 
            &partial_user.get_email().to_string(),
            tier
        ).map_err(|err| {
            AppError::domain_error(err)
        })?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email : &str) -> Result<User, AppError> {
        let user_email = UserEmail::new(email).map_err(|err| {
            AppError::domain_error(err)
        })?;

        let partial_user  = self.user_repository.get_user_by_email(&user_email).await.map_err(|err| {
            AppError::io_error(err)
        })?;
        
        let tier = self.tier_service.get_tier_by_uuid(&partial_user.get_tier().uuid).await?;
        
        let user = User::new(
            &partial_user.get_uuid().to_string(), 
            &partial_user.get_name().to_string(), 
            &partial_user.get_email().to_string(),
            tier
        ).map_err(|err| {
            AppError::domain_error(err)
        })?;

        Ok(user)
    }

    pub async fn update_user(&self, uuid : &Uuid, name : Option<String>, email : Option<String>, tier : Option<String>) -> Result<User, AppError> {
        let fetched_tier = match tier {
            Some(tier_name) => Some(self.tier_service.get_tier_by_name(&tier_name).await?),
            None => None
        };

        let partial_user = self.user_repository.update_user(uuid, name, email, fetched_tier).await.map_err(|err| {
            AppError::io_error(err)
        })?;

        let tier = self.tier_service.get_tier_by_uuid(&partial_user.get_tier().uuid).await?;
        
        let user = User::new(
            &partial_user.get_uuid().to_string(), 
            &partial_user.get_name().to_string(), 
            &partial_user.get_email().to_string(),
            tier
        ).map_err(|err| {
            AppError::domain_error(err)
        })?;

        Ok(user)
    }
}