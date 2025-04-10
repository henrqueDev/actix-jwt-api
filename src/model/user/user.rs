use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::schema::users;

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = users)]
#[diesel(treat_none_as_null = true)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub two_factor_secret: Option<String>,
    pub two_factor_recovery_code: Option<String>,
    pub two_factor_confirmed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>, 
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at:  Option<DateTime<Utc>>
}
