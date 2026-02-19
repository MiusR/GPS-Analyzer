use sqlx::PgPool;
use uuid::Uuid;

use crate::{api::model::{auth::oauth::OAuthProvider, tier::Tier, user::{User, UserEmail}}, errors::io_errors::IOError};
#[derive(Clone)]
pub struct UserRepository {
    pg_pool : PgPool
}

impl UserRepository {

    pub fn new(pg_pool : PgPool ) -> Self {
        UserRepository { pg_pool: pg_pool }
    }

    /*
        Query database @pg_pool for User with @uuid
    */
    pub async fn get_user_by_uuid(&self, uuid : &Uuid) -> Result<User, IOError> { 
        let result = sqlx::query!(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
            "#,
            &uuid
        ).fetch_one(&self.pg_pool).await;
        

        let user_row = match result {
            Ok(user_row) => user_row,
            Err(err) => {
                tracing::error!("Tried to retrieve undefined user : {}", err.to_string());
                return Err(IOError::record_not_fround("user", &err.to_string()));
            }
        };

        let user_tier = Tier { uuid: user_row.tier_uuid, name: "Unkown".to_string(), max_tracks: 0 };
        let provider = OAuthProvider::try_from(user_row.provider).map_err(|err| IOError::domain_error("parse provider",err))?;
        let user = User::new(&user_row.id.to_string(), &user_row.name, &user_row.email, user_tier, provider, user_row.provider_user_id, user_row.avatar_url).map_err(
            |err| {
                tracing::warn!("Data for user with id {} is invalid : {}", &user_row.id.to_string(), err.to_string());
                return IOError::domain_error("user internal",err)
            }
        )?;

        Ok(user)
    }

    pub async fn get_user_by_provider(&self, provider : OAuthProvider, provider_client_uuid : &str) -> Result<User, IOError> {
        let result = sqlx::query!(
            r#"
            SELECT *
            FROM users
            WHERE provider = $1 AND provider_user_id = $2 
            "#,
            &provider.to_string(),
            provider_client_uuid
        ).fetch_one(&self.pg_pool).await;
        
        let user_row = match result {
            Ok(user_row) => user_row,
            Err(err) => {
                tracing::warn!("Tried to retrieve undefined user : {}", err.to_string());
                return Err(IOError::record_not_fround("user", &err.to_string()));
            }
        };

        let user_tier = Tier { uuid: user_row.tier_uuid, name: "Unkown".to_string(), max_tracks: 0 };
        let provider = OAuthProvider::try_from(user_row.provider).map_err(|err| IOError::domain_error("parse provider",err))?;
        let user = User::new(&user_row.id.to_string(), &user_row.name, &user_row.email, user_tier, provider, user_row.provider_user_id, user_row.avatar_url).map_err(
            |err| {
                tracing::warn!("Data for user with id {} is invalid : {}", &user_row.id.to_string(), err.to_string());
                return IOError::domain_error("user internal",err)
            }
        )?;

        Ok(user)
    }

    /*
        Query database @pg_pool for User with @email
    */
    pub async fn get_user_by_email(&self, email : &UserEmail) -> Result<User, IOError> { 
        let result = sqlx::query!(
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
            &email.to_string()
        ).fetch_one(&self.pg_pool).await;
        

        let user_row = match result {
            Ok(user_row) => user_row,
            Err(err) => {
                tracing::error!("Tried to retrieve undefined user : {}", err.to_string());
                return Err(IOError::record_not_fround("user", &err.to_string()));
            }
        };

        let user_tier = Tier { uuid: user_row.tier_uuid, name: "Unkown".to_string(), max_tracks: 0 };
        let provider = OAuthProvider::try_from(user_row.provider).map_err(|err| IOError::domain_error("parse provider",err))?;
        let user = User::new(&user_row.id.to_string(), &user_row.name, &user_row.email, user_tier, provider, user_row.provider_user_id, user_row.avatar_url).map_err(
            |err| {
                tracing::warn!("Data for user with id {} is invalid : {}", &user_row.id.to_string(), err.to_string());
                return IOError::domain_error("user internal",err)
            }
        )?;

        Ok(user)
    }



    /*
        Create record in database @pg_pool of User with @user_name, @user_email, @tier
    */
    pub async fn add_user(&self, user_name : &str, user_email: &str, tier : Tier, provider : OAuthProvider, provider_client_uuid : &str, avatar_url : Option<String>) -> Result<Uuid, IOError> { 
        let user_uuid = Uuid::new_v4();

        let added_user = User::new(&user_uuid.to_string(), &user_name, &user_email, tier, provider, provider_client_uuid.to_string(), avatar_url)
        .map_err(|err| {
            tracing::error!("Failed to add user with {} and {} : {}", &user_name, &user_email, err.to_string());
            return IOError::domain_error("user", err)
        })?;
        
        let portrait= added_user.get_avatar_url().clone().unwrap_or("null".to_string());

        sqlx::query!(
            r#"
            INSERT INTO users 
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            added_user.get_uuid(),
            added_user.get_name().to_string(),
            added_user.get_email().to_string(),
            added_user.get_tier().uuid,
            added_user.get_provider().to_string(),
            added_user.get_provider_user_id(),
            portrait
        ).execute(&self.pg_pool)
        .await
        .map_err(|err| {
            tracing::warn!("Failed to add specified user {}", err.to_string());
            return IOError::record_operation("user", &err.to_string())
        })?;

        Ok(user_uuid)
    }


    /*
        Delete data from @pg_pool with id @uuid
    */
    pub async fn delete_user(&self, uuid : &Uuid) -> Result<User, IOError> { 
        let deleted_user = self.get_user_by_uuid(&uuid).await?;
        sqlx::query!(
            r#"
            DELETE FROM users 
            WHERE id = $1
            "#,
            deleted_user.get_uuid()
        ).execute(&self.pg_pool)
        .await
        .map_err(|err| {
            tracing::warn!("Failed to delete user with id [{}] : {}", uuid.to_string(), err.to_string());
            return IOError::record_operation("user", &err.to_string())
        })?;

        Ok(deleted_user)
    }


    /*
        Update data from @pg_pool of user with id @uuid to reflect new @name, @email, @tier
    */
    pub async fn update_user(&self, uuid : &Uuid, name : Option<String>, email : Option<String>, tier : Option<Tier>, avatar_url : Option<String>) -> Result<User, IOError> { 
        let old_user = self.get_user_by_uuid(&uuid).await?;

        let user_name = name.unwrap_or(old_user.get_name().to_string());
        let user_email = email.unwrap_or(old_user.get_email().to_string());
        let user_tier = match tier {
            Some(tier_name) => tier_name,
            None => old_user.get_tier().to_owned()
        };
        let user_avatar = match avatar_url {
            Some(avatar) => Some(avatar),
            None => old_user.get_avatar_url().clone()
        };

        let new_user = User::new(&uuid.to_string(), &user_name, &user_email, user_tier, old_user.get_provider().clone(), old_user.get_provider_user_id().to_string(), user_avatar)
        .map_err(|err| {
            tracing::warn!("Tried to update user with illegal paramters : {}", err.to_string());
            return IOError::domain_error("user internal", err) 
        })?;

        let portrait= new_user.get_avatar_url().clone().unwrap_or("null".to_string());

        sqlx::query!(
            r#"
            UPDATE users
            SET name = $2, email = $3, tier_uuid = $4, avatar_url = $5
            WHERE id = $1
            "#,
            &uuid,
            new_user.get_name().to_string(),
            new_user.get_email().to_string(),
            new_user.get_tier().uuid,
            portrait
        ).execute(&self.pg_pool)
        .await
        .map_err(|err| {
            tracing::warn!("Failed to update user with id {} : {}", uuid.to_string(), err.to_string());
            return IOError::record_operation("user", &err.to_string())
        })?;

        Ok(new_user)
    }

}




