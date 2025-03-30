use chrono::{DateTime, Utc};
use diesel::{prelude::{Associations, Identifiable, Insertable, Queryable}, Selectable};
use serde::Serialize;
use crate::{model::product_category::product_category::ProductCategory, schema::products};


#[derive(Queryable, Identifiable, Serialize, Insertable, Selectable, Debug, Clone, Associations)]
#[diesel(belongs_to(ProductCategory))]
#[diesel(table_name = products)]
pub struct Product {
    pub id: i32,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price: f32,
    pub weight: f32,
    pub dimension_height: f32,
    pub dimension_width: f32,
    pub dimension_depth: f32,
    pub product_category_id: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}