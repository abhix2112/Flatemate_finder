use rand::{RngCore, rngs::OsRng};
use bs58;
use chrono::{Utc, Duration};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::token::TokenResponse;

pub fn generate_token(length: usize) -> String {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    bs58::encode(bytes).into_string()
}

pub async fn create_token_pair(user_id: Uuid, db: &PgPool) -> anyhow::Result<TokenResponse> {
    let access_token = generate_token(32);
    let refresh_token = generate_token(64);
    let expires_at = Utc::now() + Duration::seconds(58);

    // Upsert token into DB
    sqlx::query!(
        r#"
        INSERT INTO user_tokens (user_id, access_token, refresh_token, expires_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id) DO UPDATE
        SET access_token = $2, refresh_token = $3, expires_at = $4
        "#,
        user_id,
        access_token,
        refresh_token,
        expires_at.naive_utc()
    )
    .execute(db)
    .await?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        expires_in: 58,
    })
}
