use chrono::{DateTime, Utc};
use diesel::{prelude::{Associations, Insertable, Queryable}, Selectable};
use serde::Serialize;
use diesel::AsChangeset;
use crate::{models::model::model::Model, schema::products};


#[derive(Queryable, Insertable, Selectable, Serialize, Debug, Clone, Associations, AsChangeset)]
#[diesel(belongs_to(Model))]
#[diesel(table_name = products)]
#[diesel(treat_none_as_null = true)]
pub struct ProductDTO {
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price: f32,
    pub weight: f32,
    pub dimension_height: f32,
    pub dimension_width: f32,
    pub dimension_depth: f32,
    pub model_id: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}