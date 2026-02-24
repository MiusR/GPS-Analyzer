use std::env;

use axum::http::Method;
use bb8_redis::{RedisConnectionManager, bb8};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{cors::build_cors_layer, model::config::Config, router::build_router, service::file_service::FileService, state::AppState};

pub mod internal;
pub mod api;
pub mod errors;

async fn connect_and_cache() -> bb8::Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(env::var("REDIS_URL").expect("Redis url not found in env!")).expect("Could not create redis connection manager");
    bb8::Pool::builder().build(manager).await.expect("Could not create connection pool")
}

async fn connect_and_migrate() -> Pool<Postgres> {
    let db_url = env::var("DATABASE_URL").expect("No database url set.");
    let pool = PgPoolOptions::new()
    .connect(&db_url).await.expect("Failed to establish connection to db pool.");
    sqlx::migrate!().run(&pool).await.expect("Migrations failed.");
    pool
}

async fn start_server(server_state : AppState) {
    let cors_methods = vec![Method::GET, Method::POST,  Method::DELETE, Method::PUT];
    let local_port = "5173";
    let app = build_router(server_state)
    .layer(build_cors_layer(local_port, cors_methods));
    // TODO : Take params from .env or even better cli 
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.expect("Adress and port must be valid and free.");
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(listener,app).await;
}


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    FileService::init().await;

    let pool = connect_and_migrate().await;
    let cache_pool = connect_and_cache().await;
    let server_state = AppState::new(Config::from_env(), pool, cache_pool);
    start_server(server_state).await;
}
