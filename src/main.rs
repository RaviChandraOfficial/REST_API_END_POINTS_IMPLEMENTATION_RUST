use axum::{
    async_trait, extract::{FromRef, FromRequestParts, State}, http::{request::Parts, StatusCode}, routing::{delete, get, post, put}, Extension, Router
};

use serde_json::Value;
use my_rest_api::handler;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::net::TcpListener;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;
use axum::{extract, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

struct AppState {}

#[tokio::main]
async fn main() {

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://people:123@localhost".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");
    println!("Connected to url:");

    // build our application with some routes

    let app = Router::new()
    .route("/get/user", get(handler::get_data))
    .route("/post/user", post(handler::post_user))
    .route("/put/user", put(handler::put_user))
    .route("/delete/user", delete(handler::delete_user)).with_state(pool);

//use redis // use sqllite 
    // run it with hyper
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

