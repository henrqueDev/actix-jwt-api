use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::users;

#[derive(Insertable, Debug, PartialEq, Serialize, Deserialize, Clone, AsChangeset)]
#[diesel(table_name = users)]
#[diesel(treat_none_as_null = true)]
pub struct UserDTO {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub user_type_id: i32,
    pub two_factor_secret: Option<String>,
    pub two_factor_recovery_code: Option<String>,
    pub two_factor_confirmed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>, 
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>
}


#[derive(Queryable, Debug, Selectable, PartialEq, Serialize, Deserialize, Clone)]
#[diesel(table_name = users)]
pub struct UserDTOMin {
    pub name: String,
    pub email: String,
    pub created_at: Option<DateTime<Utc>>, 
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>
}