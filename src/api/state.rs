
use std::sync::Arc;

use bb8_redis::{RedisConnectionManager, bb8::{self, Pool}};
use sqlx::PgPool;

use crate::api::{model::config::Config, repository::{auth_repository::AuthRepository, event_repository::EventRepository}, service::{event_service::EventService, file_service::FileService, jwt_service::JwtService, oauth_service::OAuthService, tier_service::TierService, user_service::UserService}};


pub const ACCESS_TOKEN_COOKIE: &str = "access_token";
pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
pub const DEFAULT_USER_TIER: &str = "Basic";
// TTL for access tokens: 5 minutes
pub const ACCESS_TOKEN_TTL_SECS: i64 = 5 * 60;

// TTL for refresh tokens: 1 day
pub const REFRESH_TOKEN_TTL_SECS: i64 = 24 * 60 * 60;

#[derive(Clone)]
pub struct AppState {
    config : Arc<Config>,

    cache_pool : Arc<bb8::Pool<RedisConnectionManager>>,
    db_pool : PgPool,
    
    user_service : Arc<UserService>,
    file_service : Arc<FileService>,
    tier_service : Arc<TierService>,
    jwt_service : Arc<JwtService>,
    auth_service : Arc<OAuthService>,
    event_service: Arc<EventService>
}

impl AppState { 
    pub fn new(config : Config, pg_pool : PgPool, cache : bb8::Pool<RedisConnectionManager>) -> Self {
        let shared_cache = Arc::new(cache);
        let file_service = Arc::new(FileService::new());

        let tier_service = Arc::new(TierService::new(pg_pool.clone()));
        
        let user_service = Arc::new(UserService::new(pg_pool.clone(), Arc::clone(&tier_service)));

        let jwt_service = Arc::new(JwtService::new(config.get_jwt_access_secret(), config.get_jwt_refresh_secret(), Arc::clone(&shared_cache)));
        let auth_service  = Arc::new(OAuthService::new(Arc::clone(&user_service), Arc::clone(&jwt_service), AuthRepository::new(Arc::clone(&shared_cache)) ));
        
        let event_service = Arc::new(EventService::new(EventRepository::new(pg_pool.clone()), Arc::clone(&file_service)));
        AppState {
            config : Arc::new(config),
            user_service : user_service,
            file_service : file_service,
            tier_service :  tier_service,
            jwt_service : jwt_service,
            auth_service : auth_service,
            event_service : event_service,
            cache_pool : shared_cache,
            db_pool : pg_pool.clone()
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

    pub fn get_raw_db_connection(&self) -> &PgPool {
        &self.db_pool
    }

    pub fn get_raw_cache_connection(&self) -> &Pool<RedisConnectionManager> {
        &self.cache_pool
    }

    pub fn get_jwt_service(&self) -> &JwtService {
        &self.jwt_service
    }

    pub fn get_auth_service(&self) -> &OAuthService {
        &self.auth_service
    }

    pub fn get_event_service(&self) -> &EventService {
        &self.event_service
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}