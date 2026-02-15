use std::env;

use axum::http::Method;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{controller::file_controller, cors::build_cors_layer, router::build_router, state::ServerState};

pub mod internal;
pub mod api;
pub mod errors;

async fn connect_and_migrate() -> Pool<Postgres> {
    let db_url = env::var("DATABASE_URL").expect("No database url set.");
    let pool = PgPoolOptions::new()
    .connect(&db_url).await.expect("Failed to establish connection to db pool.");
    sqlx::migrate!().run(&pool).await.expect("Migrations failed.");
    pool
}

async fn start_server(server_state : ServerState) {
    let cors_methods = vec![Method::GET, Method::POST];
    let local_port = "3000";
    let app = build_router(server_state)
    .layer(build_cors_layer(local_port, cors_methods));
    // TODO : Take params from .env or even better cli 
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
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
    file_controller::init().await;

    let pool = connect_and_migrate().await;
    let server_state = ServerState::new(pool);
    start_server(server_state).await;
}
