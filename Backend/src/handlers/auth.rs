use axum::{
    extract::{State, TypedHeader},
    headers::Authorization,
    http::StatusCode,
    Json,
};
use crate::{
    models::user::{CompleteProfileInput},
    utils::{state::AppState, token::extract_user_id_from_token},
};
use sqlx::types::Uuid;
use serde_json::json;
use chrono::Utc;

pub async fn complete_profile(
    State(state): State<AppState>,
    TypedHeader(auth_header): TypedHeader<Authorization<String>>,
    Json(input): Json<CompleteProfileInput>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Step 1: Extract user ID from secure token
    let token = auth_header.0;
    let user_id = extract_user_id_from_token(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Step 2: Insert user profile
    let _ = sqlx::query!(
        r#"
        INSERT INTO user_profiles (user_id, name, email, dob, gender, preferences, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_id,
        input.name,
        input.email,
        input.dob,
        input.gender,
        input.preferences,
        Utc::now()
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Step 3: Mark profile complete
    sqlx::query!("UPDATE users SET is_profile_complete = true WHERE id = $1", user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Step 4: Assign default role (e.g., "user")
    let role = sqlx::query!("SELECT id FROM roles WHERE name = 'user'")
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(role) = role {
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id, assigned_at) VALUES ($1, $2, $3)",
            user_id,
            role.id,
            Utc::now()
        )
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(json!({
        "status": "profile_completed",
        "user_id": user_id
    })))
}
