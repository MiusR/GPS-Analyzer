use sqlx::PgPool;
use uuid::Uuid;

use crate::{api::{io::tier_repo::{get_tier_by_name, get_tier_by_uuid}, model::user::User}, errors::io_errors::IOError};

/*
    Query database @pg_pool for User with @uuid
*/
pub async fn get_user_by_uuid(uuid : &Uuid, pg_pool : &PgPool) -> Result<User, IOError> { 
    let result = sqlx::query!(
        r#"
        SELECT *
        FROM users
        WHERE id = $1
        "#,
        &uuid
    ).fetch_one(pg_pool).await;
    

    let user_row = match result {
        Ok(user_row) => user_row,
        Err(err) => {
            tracing::error!("Tried to retrieve undefined tier {}", err.to_string());
            return Err(IOError::record_not_fround("user", &err.to_string()));
        }
    };

    let user_tier = get_tier_by_uuid(&user_row.id, &pg_pool).await?;

    let tier = User::new(&user_row.id.to_string(), &user_row.name, &user_row.email, user_tier).map_err(
        |err| {
            tracing::error!("Data for user with id {} is invalid : {}", &user_row.id.to_string(), err.to_string());
            return IOError::domain_error("user internal",err)
        }
    )?;

    Ok(tier)
}


/*
    Create record in database @pg_pool of User with @user_name, @user_email, @tier
*/
pub async fn add_user(user_name : &str, user_email: &str, tier : &str, pg_pool : &PgPool) -> Result<Uuid, IOError> { 
    let user_uuid = Uuid::new_v4();

    let added_user = User::new(&user_uuid.to_string(), &user_name, &user_email, get_tier_by_name(tier, &pg_pool).await?)
    .map_err(|err| {
        tracing::error!("Failed to add user with {} and {} : {}", &user_name, &user_email, err.to_string());
        return IOError::domain_error("user", err)
    })?;
    
    sqlx::query!(
        r#"
        INSERT INTO users 
        VALUES ($1, $2, $3, $4)
        "#,
        added_user.get_uuid(),
        added_user.get_name().to_string(),
        added_user.get_email().to_string(),
        added_user.get_tier().uuid
    ).execute(pg_pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to add specified user {}", err.to_string());
        return IOError::record_operation("user", &err.to_string())
    })?;

    Ok(user_uuid)
}


/*
    Delete data from @pg_pool with id @uuid
*/
pub async fn delete_user(uuid : &Uuid, pg_pool : &PgPool) -> Result<User, IOError> { 
    let deleted_user = get_user_by_uuid(&uuid, &pg_pool).await?;
    sqlx::query!(
        r#"
        DELETE FROM users 
        WHERE id = $1
        "#,
        deleted_user.get_uuid()
    ).execute(pg_pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to delete user with id {} : {}", uuid.to_string(), err.to_string());
        return IOError::record_operation("user", &err.to_string())
    })?;

    Ok(deleted_user)
}


/*
    Update data from @pg_pool of user with id @uuid to reflect new @name, @email, @tier
*/
pub async fn update_user(uuid : &Uuid, name : Option<String>, email : Option<String>, tier : Option<String>, pg_pool : &PgPool) -> Result<User, IOError> { 
    let old_user = get_user_by_uuid(&uuid, &pg_pool).await?;

    let user_name = name.unwrap_or(old_user.get_name().to_string());
    let user_email = email.unwrap_or(old_user.get_email().to_string());
    let user_tier = match tier {
        Some(tier_name) => get_tier_by_name(&tier_name, &pg_pool).await?,
        None => old_user.get_tier().to_owned()
    };

    let new_user = User::new(&uuid.to_string(), &user_name, &user_email, user_tier)
    .map_err(|err| {
        tracing::error!("Tried to update user with illegal paramters : {}", err.to_string());
        return IOError::domain_error("user internal", err) 
    })?;

    sqlx::query!(
        r#"
        UPDATE users
        SET name = $2, email = $3, tier_uuid = $4  
        WHERE id = $1
        "#,
        &uuid,
        new_user.get_name().to_string(),
        new_user.get_email().to_string(),
        new_user.get_tier().uuid
    ).execute(pg_pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to update user with id {} : {}", uuid.to_string(), err.to_string());
        return IOError::record_operation("user", &err.to_string())
    })?;

    Ok(new_user)
}





