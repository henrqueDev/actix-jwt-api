use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::users;

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: String, 
    pub updated_at: String,
    pub deleted_at: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserDTO {
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: String, 
    pub updated_at: String,
    pub deleted_at: String
}

