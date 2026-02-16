
use std::sync::Arc;

use sqlx::PgPool;

use crate::api::service::{file_service::FileService, tier_service::TierService, user_service::UserService};


#[derive(Clone)]
pub struct ServerState {
    user_service : UserService,
    file_service : FileService,
    tier_service : Arc<TierService>
}

impl ServerState { 
    pub fn new(pg_pool : PgPool) -> Self {
        let tier_service = Arc::new(TierService::new(pg_pool.clone()));
        let tier_clone = Arc::clone(&tier_service);
        ServerState {
            user_service : UserService::new(pg_pool.clone(), tier_clone),
            file_service : FileService::new(),
            tier_service :  tier_service
        }
    }

    pub fn get_user_service(&self) -> &UserService {
        &self.user_service
    }

    pub fn get_file_service(&self) -> &FileService {
        &self.file_service
    }

    pub fn get_tier_service(&self) -> &TierService {
        &self.tier_service
    }
}