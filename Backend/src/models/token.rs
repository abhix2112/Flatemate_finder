use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(sqlx::FromRow)]
pub struct UserToken {
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: chrono::NaiveDateTime,
}
