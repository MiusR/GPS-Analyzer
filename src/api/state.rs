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


    pub fn get_user_db(&self) -> &PgPool {
        &self.user_db
    } 
}