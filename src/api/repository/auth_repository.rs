use std::sync::Arc;

use bb8_redis::{RedisConnectionManager, bb8};



pub struct AuthRepository {
    cache : Arc<bb8::Pool<RedisConnectionManager>>
}

impl AuthRepository {
    pub fn new(cache : Arc<bb8::Pool<RedisConnectionManager>>) -> Self {
        AuthRepository { 
            cache : cache 
        }
    }

    pub fn store_auth_verifier(&self, state : String, verifier :String){
        let pool = self.cache.clone();
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

    pub async fn verify_and_consume_state(&self, state : &str) -> Option<String>{
        if let Ok(mut connection) = self.cache.clone().get().await {
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

    
}