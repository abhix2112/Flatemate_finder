use axum::{Json, extract::State};
use uuid::Uuid;
use crate::utils::state::AppState;
use crate::models::user::User;
use serde_json::json;


pub async fn signup(
    State(state): State<AppState>,
    Json(input): Json<User>,
) -> Json<serde_json::Value> {
    let id = Uuid::new_v4();
    let res = sqlx::query!(
        "INSERT INTO users (id, phone, hashed_password, created_at) VALUES ($1, $2, $3, $4)",
        id,
        input.phone,
        input.hashed_password,
        chrono::Utc::now()
     
    )
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => Json(json!({ "status": "signup_success" })),
        Err(e) => Json(json!({ "status": "error", "message": format!("{}", e) })),
    }
}
