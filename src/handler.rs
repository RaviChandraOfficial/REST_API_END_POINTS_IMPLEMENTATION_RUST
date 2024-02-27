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




/// Retrieves all sensor records from the `sensor_list` table.
///
/// This asynchronous function queries the database for all sensor records. It transforms the retrieved
/// records into a more convenient format for the response. If successful, it returns all sensor records
/// in JSON format; otherwise, it provides an appropriate error response.
///
/// # Arguments
///
/// * `State(pool)` - The database connection pool used to access the database asynchronously.
///
/// # Returns
///
/// - A successful response with HTTP status code `200 OK` and a JSON object containing all sensor records.
/// - An error response with HTTP status code `500 Internal Server Error` if there is a problem accessing the database.
///
/// # Errors
///
/// The function can return an error if there is a problem accessing the database, such as a connection issue,
/// which prevents the query from executing successfully.


// Handler for the GET request to fetch all sensor data from the database.
pub async fn get_data(
    // Extracts the PostgreSQL pool from the application state to use for database operations.
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
     // Execute a query to select all records from the sensor_list table.
    let notes = sqlx::query_as("SELECT * FROM sensor_list")
        .fetch_all(&pool) // Fetches all records asynchronously.
        .await      // Waits for the database operation to complete.
        .map_err(|e| {                 // Error handling in case the database query fails.
        // Constructs a JSON response for the error case.
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Database error: {}", e),
        });
        // Returns an internal server error status along with the JSON error message.
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;
    
// Maps each database record to a NoteModelResponse structure for the response.
    let note_responses = notes
        .iter()
        .map(|note| filter_db_record(&note))// Applies the filter_db_record function to each note.
        .collect::<Vec<NoteModelResponse>>();       // Collects the results into a vector.
 // Constructs the final JSON response with the status, total number of notes, and the note data.
    let json_response = serde_json::json!({
        "status": "success",
        "results": note_responses.len(),            // Includes the count of all notes.
        "notes": note_responses                 // Includes the serialized note data.
    });
    // Returns the JSON response with a success status.
    Ok(Json(json_response))
}




/// Retrieves a sensor record by its ID from the `sensor_list` table.
///
/// This asynchronous function takes an ID from a JSON request and queries the database for a sensor
/// record with that ID. If found, it returns the sensor record; otherwise, it provides an appropriate
/// error response.
///
/// # Arguments
///
/// * `State(pool)` - The database connection pool used to access the database asynchronously.
/// * `Json(request)` - A JSON payload containing the `id` of the sensor record to be retrieved, deserialized into a `get_id` struct.
///
/// # Returns
///
/// - A successful response with HTTP status code `200 OK` and the sensor record in JSON format if the record exists.
/// - An error response with HTTP status code `404 Not Found` if no sensor record with the given ID exists.
/// - An error response with HTTP status code `500 Internal Server Error` for any other errors encountered during database access.
///
/// # Errors
///
/// The function can return an error in two cases:
/// - If no sensor record with the provided ID exists in the database, indicating the client requested a nonexistent resource.
/// - If there is a problem accessing the database, such as a connection issue, which prevents the query from executing successfully.




// Handler for the GET request to fetch a specific sensor data entry by its ID.
pub async fn get_id_data(
     // Extracts the PostgreSQL connection pool from the application state.
    State(pool): State<PgPool>,
    // Deserialize the incoming JSON request body into a `get_id` struct.
    Json(request): Json<get_id>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
     // Extract the ID from the request body.
    let id =request.id;
     // Execute a parameterized query to select a record from the sensor_list table by ID.
    let query_result = sqlx::query_as("SELECT * FROM sensor_list WHERE id = $1")
    .bind(id)// Bind the ID to the query to prevent SQL injection.
    .fetch_one(&pool)// Fetches a single record asynchronously.
    .await;                                     // Waits for the database operation to complete.

    // Match the result of the query to handle different outcomes.
    match query_result {
        // If the query successfully finds a record, serialize it for the response.
        Ok(note) => {
            // Constructs a success response with the note data.
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": filter_db_record(&note)     // Applies filtering to the database record.
            })});
             // Returns the serialized note data with a success status.
            return Ok(Json(note_response));
        }
        // If no record is found for the given ID, return a not found error.
        Err(sqlx::Error::RowNotFound) => {
            // Constructs a fail response indicating the note was not found.
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            // Returns a 404 Not Found status with the error message.
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        // Handles other types of database errors.
        Err(e) => {
            // Constructs an error response with the error detail.
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}






/// Inserts a new sensor record into the `sensor_list` table.
///
/// This asynchronous function accepts a JSON request payload containing the details of the sensor to be inserted.
/// It validates the request, inserts the new record into the database, and returns a response indicating
/// the success or failure of the operation.
///
/// # Attributes
///
/// - `#[axum::debug_handler]`: Marks this function for special logging and debugging by Axum, 
///   providing more detailed errors if the function's signature is incorrect.
///
/// # Arguments
///
/// * `State(pool)`: The database connection pool used to access the database asynchronously.
/// * `Json(request)`: A JSON payload containing the new sensor's details (`id`, `sensor_name`, `location`, `data`).
///
/// # Returns
///
/// - A successful response with HTTP status code `200 OK` and a JSON object indicating success and the inserted record's ID and name.
/// - An error response with HTTP status code `400 Bad Request` or `500 Internal Server Error`, including a JSON object describing the error.
///
/// # Errors
///
/// The function can return errors in several scenarios:
/// - If the request payload is invalid or incomplete.
/// - If inserting the record violates database constraints, such as duplicate IDs.
/// - If there's a database access error.


// Handler for the POST request to insert a new sensor data entry into the database.
#[axum::debug_handler]
pub async fn post_data(
     // Extracts the PostgreSQL connection pool from the application state.
    State(pool): State<PgPool>,
    // Deserialize the incoming JSON request body into a `Request` struct.
    Json(request): Json<Request>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract fields from the request body.
    let id: i32 = request.id;
    let name = request.sensor_name;
    let data = request.data;
    let location = request.location;
    println!("{:?}, {:?}, ",id, type_of(&id));
    println!("{:?}, {:?} ", name, type_of(&name));
    // Execute an INSERT query to add a new record to the sensor_list table.
    let query_result =
        sqlx::query("INSERT INTO sensor_list (id,sensor_name,location, data) VALUES ($1, $2, $3, $4)")
            .bind(id.clone())
            .bind(name.to_string())
            .bind(location.to_string())
            .bind(data.to_string())
            .fetch_all(&pool)
            .await
            .map_err(|err: sqlx::Error| {
                 // Handle potential errors, particularly focusing on duplicate ID constraint violations.
                if let sqlx::Error::Database(ref db_err) = err {
                    if db_err.constraint() == Some("sensor_list") { // Check if the error is due to a primary key constraint.
                        "Duplicate ID error".to_string()
                    } else {
                        err.to_string()
                    }
                } else {
                    err.to_string()
                }
            });
     // Constructs a success response JSON with the inserted data's ID and name.
    let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "id": id,
        "name":name
    })});
     // Returns the success response with the inserted data details.
    Ok(Json(note_response))
}



fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        sensor_name: note.sensor_name.to_owned(),
        location: note.location.to_owned(),
        data: note.data.to_owned(),

    }
}


/// Updates an existing record in the `sensor_list` table with new values.
///
/// This asynchronous function receives updated sensor information through a JSON payload,
/// attempts to update the corresponding record in the database, and returns a JSON response
/// indicating success or failure.
///
/// # Arguments
///
/// * `State(pool)` - The database connection pool, wrapped in Actix's `State` for shared state access.
/// * `Json(request)` - The JSON payload containing the updated sensor data, deserialized into a `Request` struct.
///
/// # Returns
///
/// An `impl IntoResponse` which is either:
/// - A success response with HTTP status 200 and a JSON body containing the updated sensor ID and name.
/// - An error response with an appropriate HTTP status code (e.g., 500 for internal server error) and a JSON body detailing the error.
///
/// # Errors
///
/// This function returns an error if:
/// - There's a problem with the database connection or query execution (e.g., constraint violations).
/// - The specified record does not exist or cannot be updated for some reason.

pub async fn put_data(    State(pool): State<PgPool>, 
Json(request): Json<Request>,) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let id = request.id;
    let name = request.sensor_name;
    let data = request.data;
    let location = request.location;
    let query_result= sqlx::query("UPDATE sensor_list SET sensor_name=$2 , location=$3 , data=$4 WHERE id=$1")
        .bind(id.clone())
        .bind(name.to_string())
        .bind(location.to_string())
        .bind(data.to_string())
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



/// Deletes a sensor record from the `sensor_list` table based on a given ID.
///
/// This asynchronous function takes an ID from a JSON request, attempts to delete the corresponding
/// sensor record from the database, and returns an appropriate response.
///
/// # Arguments
///
/// * `State(pool)` - The database connection pool, allowing access to the database within the asynchronous function.
/// * `Json(request)` - A JSON payload containing the `id` of the sensor record to be deleted, deserialized into a `Query` struct.
///
/// # Returns
///
/// - HTTP status code `204 No Content` on successful deletion, indicating that the operation was successful and there's no additional content to send in the response.
/// - An error response with an appropriate HTTP status code (e.g., `404 Not Found` if the sensor ID does not exist in the database, or `500 Internal Server Error` for any database access issues) and a JSON body detailing the error.
///
/// # Errors
///
/// This function can result in an error response in the following scenarios:
/// - If there's an issue executing the delete operation (e.g., database connectivity problems).
/// - If the specified ID does not match any records in the database, resulting in a `404 Not Found` error.

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



 



