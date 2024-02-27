use crate::sensor::{get_id, NoteModel, NoteModelResponse, Query, Request};
use axum::response::IntoResponse;
use serde_json::{json, Value};
use sqlx::types::{uuid, Uuid};
use sqlx::{pool, query, Error, Pool};

use axum::{async_trait, extract, Extension};
use axum::extract::{FromRef, FromRequestParts, Path};
use axum::http::request::Parts;
use axum::{
    http::StatusCode,
    Json,
};

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::{Map};
use sqlx_core::HashMap;
// use sqlx::postgres::PgPool;
use tokio::sync::{MutexGuard, RwLock};
use std::io::{self, Read};
use std::num::NonZeroU8;
use structopt::StructOpt;

use axum::{extract::State};
use sqlx::postgres::{ PgPoolOptions};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;
use std::sync::Arc;
use sqlx::Postgres;



use std::any::type_name;

fn type_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}


pub async fn get_data(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let notes = sqlx::query_as("SELECT * FROM sensor_list")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let note_responses = notes
        .iter()
        .map(|note| filter_db_record(&note))
        .collect::<Vec<NoteModelResponse>>();

    let json_response = serde_json::json!({
        "status": "success",
        "results": note_responses.len(),
        "notes": note_responses
    });

    Ok(Json(json_response))
}




pub async fn get_id_data(
    State(pool): State<PgPool>,
    Json(request): Json<get_id>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id =request.id;
    let query_result = sqlx::query_as("SELECT * FROM sensor_list WHERE id = $1")
    .bind(id)
    .fetch_one(&pool)
    .await;

    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": filter_db_record(&note)
            })});

            return Ok(Json(note_response));
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}








#[axum::debug_handler]
pub async fn post_data(
    State(pool): State<PgPool>,
    Json(request): Json<Request>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id: i32 = request.id;
    let name = request.name;
    println!("{:?}, {:?}, ",id, type_of(&id));
    println!("{:?}, {:?} ", name, type_of(&name));
    let query_result =
        sqlx::query("INSERT INTO sensor_list (id,name) VALUES ($1, $2)")
            .bind(id.clone())
            .bind(name.to_string())
            .fetch_all(&pool)
            .await
            .map_err(|err: sqlx::Error| {
                if let sqlx::Error::Database(ref db_err) = err {
                    if db_err.constraint() == Some("sensor_list_pkey") {
                        "Duplicate ID error".to_string()
                    } else {
                        err.to_string()
                    }
                } else {
                    err.to_string()
                }
            });
            
    let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "id": id,
        "name":name
    })});

    Ok(Json(note_response))
}



fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        name: note.name.to_owned(),
    }
}



pub async fn put_data(    State(pool): State<PgPool>, 
Json(request): Json<Request>,) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = request.id;
    let name = request.name;
    let query_result= sqlx::query("UPDATE sensor_list SET name=$2 WHERE id=$1")
        .bind(id)
        .bind(name.clone())
        .execute(&pool)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    });

    let json_response = serde_json::json!({
        "status": "success",
        "notes":  serde_json::json!({
            "Updated_id":id,
            "Updated_name":name
        })
    });

    Ok(Json(json_response))
}


pub async fn delete_data(
    State(pool): State<PgPool>,
    Json(request): Json<Query>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = request.id;
    let query_result = sqlx::query("DELETE FROM sensor_list WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    if query_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}



fn internal_error(e: Error) -> (StatusCode, String) {
    // Handle the error, e.g., by logging and returning an appropriate HTTP status code
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", e))
}



 



