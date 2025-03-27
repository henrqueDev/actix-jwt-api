use std::fmt::Display;

use serde::Serialize;

pub mod controllers;
pub mod requests;
pub mod responses;
pub mod middleware;

#[derive(Serialize)]
pub struct GenericResponse<'a> {
    pub message: &'a str
}

#[derive(Debug, Serialize)]
pub struct GenericError<'a> {
    pub message: &'a str,
    pub error: &'a str
}

// Retornando string na resposta ao invés de JSON (Revisar)
impl Display for GenericError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ \"message\":{:#?}, \"error\":{:#?} }}", self.message, self.error)
    }
}