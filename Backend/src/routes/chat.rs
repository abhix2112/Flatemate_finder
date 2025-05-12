use chrono::{DateTime, NaiveDateTime, Utc};
use axum::{extract::{State, Path}, Json, http::StatusCode};
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::middleware::auth::AuthenticatedUser;
use crate::utils::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
pub struct ChatSession {
    pub id: Uuid,
    pub sender_id: Option<Uuid>,
    pub receiver_id: Option<Uuid>,
    pub listing_id: Option<Uuid>,
    pub request_id: Option<Uuid>,
    pub chat_dropped: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub last_message_at: Option<NaiveDateTime>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub chat_session_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub chat_session_id: Uuid,
    pub content: String,
}

#[derive(Deserialize)]
pub struct DropChatRequest {
    pub chat_session_id: Uuid,
}

pub async fn send_message(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, .. }: AuthenticatedUser,
    Json(payload): Json<SendMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    let session = sqlx::query_as!(ChatSession,
        "SELECT * FROM chat_sessions WHERE id = $1",
        payload.chat_session_id
    ).fetch_one(&state.db).await.map_err(|_| StatusCode::NOT_FOUND)?;

    if session.chat_dropped.unwrap_or(false) {
        return Err(StatusCode::FORBIDDEN);
    }

    let msg_id = Uuid::new_v4();
    let now = Utc::now().naive_utc();

    sqlx::query!(
        "INSERT INTO chat_messages (id, chat_session_id, sender_id, message)
         VALUES ($1, $2, $3, $4)",
        msg_id,
        payload.chat_session_id,
        user_id,
        payload.content
    ).execute(&state.db).await.map_err(|e| {
        eprintln!("Send message error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "UPDATE chat_sessions SET last_message_at = $1 WHERE id = $2",
        now,
        payload.chat_session_id
    ).execute(&state.db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}







pub async fn get_conversation(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Path(chat_session_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    let session = sqlx::query_as!(
        ChatSession,
        "SELECT * FROM chat_sessions WHERE id = $1",
        chat_session_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch chat session: {:?}", e);
        StatusCode::NOT_FOUND
    })?;

    // Properly handle the Option<Uuid> fields
    let has_access = match (session.sender_id, session.receiver_id) {
        (Some(sender_id), Some(receiver_id)) => {
            user_id == sender_id || user_id == receiver_id
        },
        _ => false
    };

    if !has_access {
        return Err(StatusCode::FORBIDDEN);
    }

    // Properly handle the Option<bool> field
    if let Some(true) = session.chat_dropped {
        return Err(StatusCode::FORBIDDEN);
    }

    let messages = sqlx::query!(
        r#"
        SELECT 
            id as "id: Uuid",
            chat_session_id as "chat_session_id: Uuid",
            sender_id as "sender_id: Uuid",
            message as "content: String",
            created_at as "created_at: NaiveDateTime"
        FROM chat_messages
        WHERE chat_session_id = $1
        ORDER BY created_at ASC
        "#,
        chat_session_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Fetch messages error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Convert to the Message struct
    let messages: Vec<Message> = messages
        .into_iter()
        .map(|row| Message {
            id: row.id,
            chat_session_id: row.chat_session_id.unwrap(),
            sender_id: row.sender_id.unwrap(),
            content: row.content,
            created_at: row.created_at.unwrap(),
        })
        .collect();

    Ok(Json(messages))
}