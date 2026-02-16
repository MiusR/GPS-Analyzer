use sqlx::PgPool;
use uuid::Uuid;

use crate::{api::model::tier::Tier, errors::io_errors::IOError};


#[derive(Clone)]
pub struct TierRepository {
    pg_pool : PgPool
}

impl TierRepository {
    pub fn new(pg_pool : PgPool) -> Self {
        TierRepository { pg_pool }
    }

    /*
        Query data base @pg_pool for Tier by @name
    */
    pub async fn get_tier_by_name(&self, name : &str) -> Result<Tier, IOError> { 
        let result = sqlx::query!(
            r#"
            SELECT *
            FROM tiers
            WHERE name = $1
            "#,
            &name
        ).fetch_one(&self.pg_pool).await;
        

        let tier_row = match result {
            Ok(tier_row) => tier_row,
            Err(err) => {
                tracing::error!("Tried to retrieve undefined tier {}", err.to_string());
                return Err(IOError::record_not_fround("tier", &err.to_string()));
            }
        };

        let tier = Tier {
            uuid : tier_row.id, 
            max_tracks : tier_row.max_tracks, 
            name : tier_row.name 
        };

        Ok(tier)
    }

    /*
        Query data base @pg_pool for Tier by @uuid
    */
    pub async fn get_tier_by_uuid(&self, uuid : &Uuid) -> Result<Tier, IOError> { 
        let result = sqlx::query!(
            r#"
            SELECT *
            FROM tiers
            WHERE id = $1
            "#,
            &uuid
        ).fetch_one(&self.pg_pool).await;
        

        let tier_row = match result {
            Ok(tier_row) => tier_row,
            Err(err) => {
                tracing::error!("Tried to retrieve undefined tier {}", err.to_string());
                return Err(IOError::record_not_fround("tier", &err.to_string()));
            }
        };

        let tier = Tier {
            uuid : tier_row.id, 
            max_tracks : tier_row.max_tracks, 
            name : tier_row.name 
        };

        Ok(tier)
    }




    /*
        Insert data into base @pg_pool with for Tier with @name and @max_files
    */
    pub async fn create_tier(&self, name : &str, max_files : i32) -> Result<Uuid, IOError> { 
        let tier_uuid = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO tiers 
            VALUES ($1, $2, $3)
            "#,
            &tier_uuid,
            &name,
            max_files
        ).execute(&self.pg_pool)
        .await.map_err(|err| {
            tracing::error!("Failed to add specified tier {}", err.to_string());
            IOError::record_operation("database", &err.to_string())
        })?;
        
        Ok(tier_uuid)
    }


}

