use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchNearbyInput {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
pub struct NearbyListing {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub formatted_address: Option<String>,
    pub is_active: bool,
    pub distance: f64, // calculated using Haversine
}
