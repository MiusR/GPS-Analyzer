use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::model::tier::Tier;

/*
    Query data base @pg_pool with for Tier with @name
*/
pub async fn get_tier_by_name(name : &str, pg_pool : &PgPool) -> Result<Tier, StatusCode> { 
    let result = sqlx::query!(
        r#"
        SELECT *
        FROM tiers
        WHERE name = $1
        "#,
        &name
    ).fetch_one(pg_pool).await;
    

    let tier_row = match result {
        Ok(tier_row) => tier_row,
        Err(err) => {
            tracing::error!("Tried to retrieve undefined tier {}", err.to_string());
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let tier = Tier {
        id : tier_row.id, 
        max_tracks : tier_row.max_tracks, 
        name : tier_row.name 
    };

    Ok(tier)
}



/*
    Insert data into base @pg_pool with for Tier with @name and @max_files
*/
pub async fn create_tier(name : &str, max_files : i32, pg_pool : &PgPool) -> Result<Uuid, StatusCode> { 
    let tier_uuid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO tiers 
        VALUES ($1, $2, $3)
        "#,
        &tier_uuid,
        &name,
        max_files
    ).execute(pg_pool)
    .await.map_err(|err| {
        tracing::error!("Failed to add specified tier {}", err.to_string());
        return StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(tier_uuid)
}