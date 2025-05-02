use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDate, DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub phone: String,
    pub hashed_password: String,
    pub created_at: DateTime<Utc>,
    pub is_profile_complete: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: Option<String>,
    pub email: Option<String>,
    pub dob: Option<NaiveDate>,
    pub gender: Option<String>,
    pub preferences: Option<serde_json::Value>, // JSON field
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

// Input payload to complete profile
#[derive(Debug, Deserialize)]
pub struct CompleteProfileInput {
    pub name: String,
    pub email: String,
    pub dob: Option<NaiveDate>,
    pub gender: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

// This will be used to return token + next step info
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub is_profile_complete: bool,
}
