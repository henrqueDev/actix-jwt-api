use chrono::{DateTime, Utc};
use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use serde::Serialize;
use crate::schema::models;

#[derive(Queryable, Insertable, Selectable, Serialize, Debug, Clone, AsChangeset)]
#[diesel(table_name = models)]
#[diesel(treat_none_as_null = true)]
pub struct ModelDTO {
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>> 
}