use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct OtpLog {
    pub id: Uuid,
    pub phone: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub attempts: i32,
}

#[derive(Debug, Deserialize)]
pub struct Sendotp {
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct OTPInput {
    pub phone: String,
    pub otp: String,
}
