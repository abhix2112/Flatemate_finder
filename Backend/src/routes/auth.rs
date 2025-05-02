use axum::{extract::State, Json};
use serde_json::json;
use sqlx::types::Uuid;

use crate::{
    utils::{state::AppState, token::{create_token_pair, generate_token}},
    models::token::TokenResponse,
};

#[derive(serde::Deserialize)]
pub struct RefreshInput {
    refresh_token: String,
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(input): Json<RefreshInput>,
) -> Json<serde_json::Value> {
    let result = sqlx::query!(
        r#"
        SELECT user_id FROM user_tokens 
        WHERE refresh_token = $1
        "#,
        input.refresh_token
    )
    .fetch_optional(&state.db)
    .await;

    if let Ok(Some(record)) = result {
        let tokens = create_token_pair(record.user_id, &state.db)
            .await
            .expect("token regeneration failed");

        Json(json!({
            "status": "success",
            "tokens": tokens
        }))
    } else {
        Json(json!({
            "status": "error",
            "message": "Invalid or expired refresh token"
        }))
    }
}
