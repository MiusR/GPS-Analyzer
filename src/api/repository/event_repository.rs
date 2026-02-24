use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{api::model::racing_event::RacingEvent, errors::io_errors::IOError};


#[derive(Clone)]
pub struct EventRepository {
    pg_pool : PgPool
}


impl EventRepository {
    pub fn new(pg_pool : PgPool) -> Self {
        EventRepository { pg_pool }
    }

    // TODO maybe move all these queries into sql functions or views
    /*
        Query data base @pg_pool for RacingEvent owned by user with @uuid
    */
    pub async fn get_events_by_owner(&self, user_uuid: &Uuid) -> Result<Vec<RacingEvent>, IOError> { 
        let rows = sqlx::query!(
            r#"
            SELECT racing_events.id, racing_events.event_name, racing_events.created_at
            FROM user_events ue
            INNER JOIN racing_events 
            ON racing_events.id = ue.racing_id
            WHERE ue.id = $1
            "#,
            user_uuid
        ).fetch_all(&self.pg_pool).await
        .map_err(|err| {
            tracing::error!("Database error: {}", err);
            IOError::record_not_fround("event", &err.to_string())
        })?;

        // Map the vector of database rows into a vector of RacingEvent structs
        let events = rows.into_iter().map(|row| {
            RacingEvent {
                uuid: row.id,  
                event_name: row.event_name,
                created_at:  DateTime::from_naive_utc_and_offset(row.created_at, Utc)
            }
        }).collect();

        Ok(events)
    }

    /*
        Query data base @pg_pool for racing event by @uuid
    */
    pub async fn get_event_by_uuid(&self, uuid : &Uuid) -> Result<RacingEvent, IOError> { 
        let result = sqlx::query!(
            r#"
            SELECT *
            FROM racing_events
            WHERE id = $1
            "#,
            &uuid
        ).fetch_one(&self.pg_pool).await;
        

        let event_row = match result {
            Ok(event_row) => event_row,
            Err(err) => {
                tracing::error!("Tried to retrieve undefined event {}", err.to_string());
                return Err(IOError::record_not_fround("event", &err.to_string()));
            }
        };

        let event = RacingEvent {
            uuid: event_row.id,
            event_name : event_row.event_name,
            created_at : DateTime::from_naive_utc_and_offset(event_row.created_at, Utc)
        };

        Ok(event)
    }




    /*
        Insert data into base @pg_pool with for Tier with @name and @max_files
    */
    pub async fn create_event(&self, name : &str, user_uuid: &Uuid) -> Result<Uuid, IOError> { 
        let event_uuid = Uuid::new_v4();
        let utc_now = Utc::now().naive_utc();
        sqlx::query!(
            r#"
            INSERT INTO racing_events
            VALUES ($1, $2, $3)
            "#,
            &event_uuid,
            &name,
            &utc_now
        ).execute(&self.pg_pool)
        .await.map_err(|err| {
            tracing::error!("Failed to add specified event {}", err.to_string());
            IOError::record_operation("database", &err.to_string())
        })?;

        let relation_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO user_events
            VALUES ($1, $2, $3)
            "#,
            &relation_id,
            &user_uuid,
            &event_uuid
        ).execute(&self.pg_pool)
        .await.map_err(|err| {
            tracing::error!("Failed to add specified event relation {}", err.to_string());
            IOError::record_operation("database", &err.to_string())
        })?;
        
        Ok(event_uuid)
    }


}

