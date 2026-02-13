use std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone)]
pub struct ServerState {
    user_db : Arc<PgPool>
}

impl ServerState { 
    pub fn new(pg_pool : PgPool) -> Self {
        ServerState {  
            user_db : Arc::new(pg_pool)
        }
    }
}