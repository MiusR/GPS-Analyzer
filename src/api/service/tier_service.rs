use sqlx::PgPool;
use uuid::Uuid;

use crate::{api::{model::tier::Tier, repository::tier_repository::TierRepository}, errors::app_error::AppError};

#[derive(Clone)]
pub struct TierService {
    tier_repository : TierRepository
}

impl TierService {
    pub fn new(pg_pool : PgPool) -> Self {
        TierService { tier_repository: TierRepository::new(pg_pool) }
    }


    pub async fn get_tier_by_name(&self, name : &str) -> Result<Tier, AppError> {
        self.tier_repository.get_tier_by_name(name).await.map_err(|err| {
            AppError::io_error(err)
        })
    }

    pub async fn get_tier_by_uuid(&self, uuid : &Uuid) -> Result<Tier, AppError> {
        self.tier_repository.get_tier_by_uuid(uuid).await.map_err(|err| {
            AppError::io_error(err)
        })
    }

    pub async fn create_tier(&self, name : &str, max_files : i32) -> Result<Uuid, AppError> {
        self.tier_repository.create_tier(name, max_files).await.map_err(|err| {
            AppError::io_error(err)
        })
    }
}