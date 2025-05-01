use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub phone: String,
    pub full_name: String,
    pub email: String,
    pub password_hash: String,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct PhoneInput {
    pub phone: String,
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Sendotp {
    pub phone: String
}

#[derive(Serialize, Deserialize)]
pub struct OTPInput {
    pub phone: String,
    pub otp: String,
} 