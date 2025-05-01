use axum::{Json, extract::State};
use uuid::Uuid;
use crate::utils::state::AppState;
use crate::models::PhoneInput;
use serde_json::json;

pub async fn signup(
    State(state): State<AppState>,
    Json(input): Json<PhoneInput>,
) -> Json<serde_json::Value> {
    let id = Uuid::new_v4();
    let res = sqlx::query!(
        "INSERT INTO users (id, full_name, phone, email, password_hash, is_verified) VALUES ($1, $2, $3, $4, $5, $6)",
        id,
        input.full_name,
        input.phone,
        input.email,
        input.password,
        false
    )
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => Json(json!({ "status": "signup_success" })),
        Err(e) => Json(json!({ "status": "error", "message": format!("{}", e) })),
    }
}
