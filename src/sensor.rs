use std::sync::{Arc, Mutex};
use axum::{async_trait, extract::rejection::StringRejection, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;



// define a struct for the request body
#[derive(Deserialize)]
pub struct Request {
    pub id: i32,
    pub sensor_name: String,
    pub data: String,
    pub location: String,
}

// // define a struct for the response body
// #[derive(Serialize)]
// pub struct Response {
//     pub id: i32,
//     pub name: String,
// }


// define a struct for the query parameters
#[derive(Deserialize)]
pub struct Query {
    pub id: i32,
}

// // define a struct for the database record
// #[derive(sqlx::FromRow, serde::Serialize)]
// pub struct Record {
//     pub id: i32,
//     pub name: String,
// }



#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct NoteModel {
    pub id: i32,
    pub sensor_name: String,
    pub location :String,
    pub data :String,
}


#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct NoteModelResponse {
    pub id: i32,
    pub sensor_name: String,
    pub location: String,
    pub data : String
}


#[derive(Deserialize, Serialize)]
pub struct get_id {
    pub id:i32
}