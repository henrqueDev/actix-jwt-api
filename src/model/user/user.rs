use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::schema::users;

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: Option<DateTime<Utc>>, 
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at:  Option<DateTime<Utc>>
}
