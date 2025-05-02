use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::utils::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub from: Uuid,
    pub to: Uuid,
    pub message: String,
}

pub async fn send_message(
    State(state): State<AppState>,
    Json(payload): Json<ChatMessage>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("INSERT INTO messages (id, sender_id, receiver_id, content) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::new_v4())
        .bind(payload.from)
        .bind(payload.to)
        .bind(payload.message)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}

pub async fn get_chat(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<ChatMessage>>, StatusCode> {
    let messages = sqlx::query_as::<_, ChatMessage>(
        "SELECT sender_id as from, receiver_id as to, content as message FROM messages WHERE sender_id = $1 OR receiver_id = $1"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages))
}