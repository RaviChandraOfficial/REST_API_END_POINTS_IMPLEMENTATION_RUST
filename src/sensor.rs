use axum::{async_trait, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
// use async_trait::async_trait;
use sqlx::FromRow;
// use sqlx::sqlite::SqlitePool;
use sqlx::sqlx_macros::expand_query;



#[derive(Debug, Clone,serde::Deserialize, serde::Serialize)]
pub struct Sensor {
    pub id: u32,
    pub name: String,
    pub location: String,
    pub data: String,
}

#[derive(Debug, Clone,serde::Deserialize, serde::Serialize)]
pub struct post_data{
    pub id:u32,
    pub name: String,

}

#[derive(Debug, Clone,serde::Deserialize, serde::Serialize)]
pub struct get_data{
    pub id:u32,
    pub name: String,
    pub location: String,
    pub data: String,
}


// define a struct for the request body
#[derive(Deserialize)]
pub struct Request {
    pub id: i32,
    pub name: String,
}

// define a struct for the response body
#[derive(Serialize)]
pub struct Response {
    pub id: i32,
    pub name: String,
}


// define a struct for the query parameters
#[derive(Deserialize)]
pub struct Query {
    pub id: i32,
}

// define a struct for the database record
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Record {
    pub id: i32,
    pub name: String,
}



#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct NoteModel {
    pub id: i32,
    pub name: String,
}


#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct NoteModelResponse {
    pub id: i32,
    pub name: String,}

