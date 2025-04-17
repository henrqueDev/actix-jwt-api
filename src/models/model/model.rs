use chrono::{DateTime, Utc};
use diesel::{prelude::{Identifiable, Queryable}, Selectable};
use crate::schema::models;

#[derive(Queryable, Debug, Clone, Selectable, Identifiable)]
#[diesel(table_name = models)]
#[diesel(treat_none_as_null = true)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>> 
}