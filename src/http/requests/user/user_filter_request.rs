use chrono::{DateTime, Utc};
use serde::Deserialize;


#[derive(Deserialize)]
pub struct UserFilterRequest {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub id: Option<i32>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub created_at: Option<DateTime<Utc>>, 
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>
}