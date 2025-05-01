use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PhoneInput {
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub password: String,
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