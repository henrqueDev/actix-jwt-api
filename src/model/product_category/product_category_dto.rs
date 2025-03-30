use chrono::{DateTime, Utc};
use diesel::{prelude::Queryable, Selectable};
use crate::schema::product_categories;

#[derive(Queryable, Debug, Clone, Selectable)]
#[diesel(table_name = product_categories)]
pub struct ProductCategoryDTO {
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>> 
}