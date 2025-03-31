use chrono::{DateTime, Utc};
use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use serde::Serialize;
use crate::schema::product_categories;

#[derive(Queryable, Insertable, Selectable, Serialize, Debug, Clone, AsChangeset)]
#[diesel(table_name = product_categories)]
#[changeset_options(treat_none_as_null = "true")]
pub struct ProductCategoryDTO {
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>> 
}