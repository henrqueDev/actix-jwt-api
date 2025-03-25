use serde::Serialize;

pub mod controllers;
pub mod requests;
pub mod responses;
pub mod middleware;

#[derive(Serialize)]
pub struct GenericResponse<'a> {
    pub message: &'a str
}

#[derive(Serialize)]
pub struct GenericError<'a> {
    pub message: &'a str,
    pub error: &'a str
}