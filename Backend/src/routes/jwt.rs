use axum::{Json, extract::State};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::utils::state::AppState;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

fn get_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

pub fn create_token(sub: &str, role: &str) -> String {
    let exp = (Utc::now() + Duration::days(1)).timestamp() as usize;
    let claims = Claims {
        sub: sub.to_string(),
        exp,
        role: role.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_secret().as_bytes())
    ).unwrap()
}

pub async fn refresh_token(State(_state): State<AppState>) -> Json<serde_json::Value> {
    // To be implemented: extract refresh token from cookie, validate, and issue new token
    Json(json!({ "message": "token refreshed" }))
}