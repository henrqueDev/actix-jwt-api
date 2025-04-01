use chrono::{DateTime, Utc};
use diesel::{prelude::{Identifiable, Queryable}, Selectable};
use crate::schema::product_categories;

#[derive(Queryable, Debug, Clone, Selectable, Identifiable)]
#[diesel(table_name = product_categories)]
#[diesel(treat_none_as_null = true)]
pub struct ProductCategory {
    pub id: i32,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>> 
}