
use std::sync::Arc;

use bb8_redis::{RedisConnectionManager, bb8::{self, Pool, RunError}};
use chrono::{DateTime, Utc};
use redis::{AsyncCommands, RedisError};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{api::{model::{auth::oauth::OAuthProvider, config::Config, dto::jwt_request::RefreshTokenRecord, user::User}, service::{file_service::FileService, jwt_service::JwtService, oauth_service::OAuthService, tier_service::TierService, user_service::UserService}}, errors::{app_error::AppError, io_errors::IOError}};


#[derive(Clone)]
pub struct AppState {
    config : Arc<Config>,

    cache_pool : bb8::Pool<RedisConnectionManager>,
    db_pool : PgPool,
    
    user_service : UserService,
    file_service : FileService,
    tier_service : Arc<TierService>,
    jwt_service : Arc<JwtService>,
    auth_service : Arc<OAuthService>
}

impl AppState { 
    pub fn new(config : Config, pg_pool : PgPool, cache : bb8::Pool<RedisConnectionManager>) -> Self {
        let tier_service = Arc::new(TierService::new(pg_pool.clone()));
        let tier_clone = Arc::clone(&tier_service);
        
        let jwt_service = Arc::new(JwtService::new(config.get_jwt_access_secret(), config.get_jwt_refresh_secret()));
        let auth_service  = Arc::new(OAuthService {});
        AppState {
            config : Arc::new(config),
            user_service : UserService::new(pg_pool.clone(), tier_clone),
            file_service : FileService::new(),
            tier_service :  tier_service,
            jwt_service : jwt_service,
            auth_service : auth_service,
            cache_pool : cache,
            db_pool : pg_pool.clone()
        }
    }

    pub fn upsert_user(
        &self,
        provider: OAuthProvider,
        provider_user_id: String,
        email: Option<String>,
        name: Option<String>,
        avatar_url: Option<String>,
    ) -> User {
        let key = (provider.clone(), provider_user_id.clone());

        if let Some(existing) = self.user_service.get(&key) {
            return existing.clone();
        }

        let user = User::new(provider, provider_user_id, email, name, avatar_url);
        self.users.insert(key, user.clone());
        self.users_by_id.insert(user.id, user.clone());
        user
    }

    //TODO Move this into jwt service -------------------------------------------------------------------------

    pub async fn store_refresh_token(
        &self,
        user_id: Uuid,
        jti: String,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let record = RefreshTokenRecord {
            user_id,
            jti: jti.clone(),
            expires_at,
            revoked: false,
        };
        let serialized = serde_json::to_string(&record)
            .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?;
        
        let ttl = (expires_at - Utc::now()).num_seconds().max(0) as u64;
        let pool = self.cache_pool.clone();
        
        tokio::spawn(async move {
            if let Ok(mut conn) = pool.get().await {
                let _ = redis::cmd("SET")
                    .arg(&jti)
                    .arg(&serialized)
                    .arg("EX")
                    .arg(ttl)
                    .query_async::<()>(&mut *conn)
                    .await;
            }
        });
        Ok(())
    }


    pub async fn validate_refresh_token(&self, jti: &str) -> Option<RefreshTokenRecord> {
        let mut conn = self.cache_pool.get().await.ok()?;
        let bytes = redis::cmd("GET")
            .arg(jti)
            .query_async::<Option<Vec<u8>>>(&mut *conn)
            .await
            .ok()??;
        
        let record: RefreshTokenRecord = serde_json::from_slice(&bytes).ok()?;
        if record.revoked || record.expires_at <= Utc::now() {
            return None;
        }

        // GLOBAL CACHE INVALIDATION 
        let revoke_key = format!("revoked_all:{}", record.user_id);
        if let Ok(Some(ts)) = redis::cmd("GET")
            .arg(&revoke_key)
            .query_async::<Option<i64>>(&mut *conn)
            .await
        {
            let revoked_at = DateTime::from_timestamp(ts, 0)?;
            if record.expires_at <= revoked_at {
                return None;
            }
        }

        Some(record)
    }

    pub async fn revoke_refresh_token(&self, jti: &str) -> Result<(), AppError> {
        let mut conn = self.cache_pool.get().await
            .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?;
        
        let bytes = redis::cmd("GET")
            .arg(jti)
            .query_async::<Option<Vec<u8>>>(&mut *conn)
            .await
            .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?;
        
        if let Some(bytes) = bytes {
            let mut record: RefreshTokenRecord = serde_json::from_slice(&bytes)
                .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?;
            record.revoked = true;
            let serialized = serde_json::to_string(&record)
                .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?;
            // Preserve remaining TTL
            let ttl = (record.expires_at - Utc::now()).num_seconds().max(0) as u64;
            redis::cmd("SET")
                .arg(jti)
                .arg(&serialized)
                .arg("EX")
                .arg(ttl)
                .query_async::<()>(&mut *conn)
                .await
                .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?;
        }
        Ok(())
    }


    pub async fn revoke_all_user_tokens(&self, user_id: &Uuid) -> Result<(), AppError> {
        let mut conn = self.cache_pool.get().await
            .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?; 
        
        let key = format!("revoked_all:{}", user_id);
        let now = Utc::now().timestamp();
        
        // TODO : maybe add this to postgress as a marker of invalidation in case of critical failiure or just issue a global invalidation token on startup
        redis::cmd("SET")
            .arg(&key)
            .arg(now)
            .arg("EX")
            .arg(60 * 60 * 24 * 30)// Period where the refresh is kept
            .query_async::<()>(&mut *conn)
            .await
            .map_err(|e| AppError::io_error(IOError::record_not_fround("redis", &e.to_string())))?; 
        
        Ok(())
    }

    // TODO Move this to auth service --------------------------------------------------------------------------
    /*
        Store auth in cache with autodelete
    */
    pub async fn store_oauth_state(&self, state: String, verifier : String) {
        let pool = self.cache_pool.clone();
        
        tokio::spawn(async move {
            if let Ok(mut conn) = pool.get().await {
                let _ = redis::cmd("SET")
                    .arg(&state)
                    .arg(&verifier)
                    .arg("EX")
                    .arg(60 * 10) // 10 mins
                    .query_async::<()>(&mut *conn)
                    .await;
            }else {
                tracing::error!("Could not store state {} to redis.", &state);
            }
        });
    }

    /*
        Verify if the value is in cache and delete if it is.
    */
    pub async fn validate_and_consume_oauth_state(&self, state: &str) -> Option<String> {
        if let Ok(mut connection) = self.cache_pool.clone().get().await {
        let result = redis::cmd("GETDEL")
            .arg(state)
            .query_async::<Option<String>>(&mut *connection)
            .await;
            return result.ok().flatten();
        } else {
            tracing::error!("Could not retrieve from redis {}.", &state);
        }
        None
    }
    
    // -----------------------------------------------------------------------------------------------------------

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

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}