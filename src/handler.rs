use crate::sensor::{self, post_data, NoteModel, NoteModelResponse, Query, Record, Request, Response, Sensor};
use axum::response::IntoResponse;
use serde_json::{json, Value};
use sqlx::types::uuid;
use sqlx::{Error, Pool};
// use wrap::http::StatusCode;
use axum::{async_trait, extract, Extension};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::{
    http::StatusCode,
    Json,
};
use axum::{extract::State};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::time::Duration;


use serde::{Deserialize, Serialize};

use sqlx::Postgres;

pub async fn using_connection_pool_extractor(
    State(pool): State<PgPool>,
) -> Result<String, (StatusCode, String)> {
    let x=sqlx::query_scalar("select * from sensor_var_char ")
        .fetch_one(&pool)
        .await
        .map_err(internal_error);
    x
}

// define a handler function for the get method


pub async fn get_data(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let notes = sqlx::query_as(
        "SELECT * FROM sensor_json")
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





pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Rust CRUD API Example with Axum Framework and MySQL";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}






// pub async fn get_record(
//     query: extract::Query<Query>,
//     pool: extract::Extension<Pool<Postgres>>,
// ) -> Result<Json<Record>, String> {
//     // get the id from the query parameters
//     let id = query.id;
//     // execute a SQL query to fetch the record by id
//     let record = sqlx::query_as::<_, Record>("select * from sensor_var_char where id = $1")
//         .bind(id)
//         .fetch_one(&*pool)
//         .await
//         .map_err(|e| e.to_string())?;

//     // return the record as JSON
//     Ok(Json(record))
// }

// define a handler function for the post method
pub async fn post_method(
    Json(request): Json<Request>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<Json<Response>, (StatusCode, Json<serde_json::Value>)> {
    let id = request.id;
    let name = request.name;

    // execute a SQL query to insert a new record and return the id
    let x = sqlx::query("insert into  sensor_json(id name) values ($1, $2) returning id")
        .bind(name.clone())
        .fetch_one(&pool)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    // create a response body with the id, name, and age
    let response = Response { id, name };

    // return the response body as JSON
    Ok(Json(response))
}




pub async fn create_note_handler(
    Json(request): Json<Request>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = request.id;
    let name = request.name;
    let query_result =
        sqlx::query(r#"INSERT INTO notes (id,title,content,category) VALUES (?, ?, ?, ?)"#)
            .bind(user_id.clone())
            .bind(name.to_string())
            .execute(&pool)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Note with that title already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }

    let note = sqlx::query_as( r#"SELECT * FROM notes WHERE id = ?"#)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "note": filter_db_record(&note)
    })});

    Ok(Json(note_response))
}





pub async fn post_user(
    Json(request): Json<Request>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = request.id;
    let name = request.name;
    let query_result =
        sqlx::query("INSERT INTO sensor_list (id,name) VALUES (id, name)")
            .bind(id.clone())
            .bind(name.to_string())
            .fetch_one(&pool)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Note with that title already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }

    let note = sqlx::query_as( r#"SELECT * FROM sensor_list WHERE id = ?"#)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    // let response = Response { id, name };
    let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "note": filter_db_record(&note)
    })});

    Ok(Json(note_response))
}
// async fn using_connection_extractor(
//     DatabaseConnection(mut conn): DatabaseConnection,
// ) -> Result<String, (StatusCode, String)> {
//     sqlx::query_scalar("select 'hello world from pg'")
//         .fetch_one(&mut *conn)
//         .await
//         .map_err(internal_error)
// }


fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        name: note.name.to_owned(),
    }
}



// // this argument tells axum to parse the request body
// // as JSON into a `CreateUser` type
// pub async fn create_user(Json(payload): Json<CreateUser>,) -> (StatusCode, Json<User>) {
//     // insert your application logic here
//     let user = User {
//         id: 3,
//         username: payload.username,
//     };
    
//     // this will be converted into a JSON response
//     // with a status code of `201 Created`
//     (StatusCode::CREATED, Json(user))
// }
   

// define a handler function for the put method
pub async fn update_record(
    Json(request): Json<Request>,
    Extension(pool): Extension<Pool<Postgres>>,
) -> Result<Json<Response>, String> {
    // get the id, name, and age from the request body
    let id = request.id;
    let name = request.name;


    // execute a SQL query to update the record by id
    sqlx::query("update records set name = $1, age = $2 where id = $3")
        .bind(&name)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // create a response body with the id, name, and age
    let response = Response { id, name};

    // return the response body as JSON
    Ok(Json(response))
}

// define a handler function for the delete method
pub async fn delete_record(
    query: extract::Query<Query>,
    pool: extract::Extension<Pool<Postgres>>,
) -> Result<String, String> {
    // get the id from the query parameter
    let id = query.id;

    // execute a SQL query to delete the record by id
    sqlx::query("delete from records where id = $1")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    // return a success message
    Ok(format!("Record with id {} deleted successfully", id))
}

// pub async fn using_connection_extractor(
//     DatabaseConnection(mut conn): DatabaseConnection,
// ) -> Result<String, (StatusCode, String)> {
//     sqlx::query_scalar("select * from sensor_var_char ")
//         .fetch_one(&mut *conn)
//         .await
//         .map_err(internal_error)
// }

pub struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let conn = pool.acquire().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

fn internal_error(e: Error) -> (StatusCode, String) {
    // Handle the error, e.g., by logging and returning an appropriate HTTP status code
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", e))
}


pub async fn put_user(Json(payload): Json<post_data>)  ->  Json<post_data>{
    // handle the PUT request here
    // for example, update an item in a database
    // println!("Updating item: {:?}", payload);
    let user= post_data{
        id:payload.id,
        name:payload.name,

    };
    // Respond with a status code and a message
    Json(user)
}

 


pub async fn using_connection_extractor(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&mut *conn)
        .await
        .map_err(internal_error)
}


