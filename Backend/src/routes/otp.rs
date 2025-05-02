use axum::{Json, extract::State};
use serde_json::json;
use chrono::{Duration, Utc};
use crate::utils::state::AppState;
use crate::utils::twilio::{send_otp_via_twilio_verify, verify_otp_via_twilio};
use crate::models::otp::{OTPInput, Sendotp};
use crate::utils::token::{create_token_pair};

pub async fn send_otp(
    State(state): State<AppState>,
    Json(payload): Json<Sendotp>,
) -> Json<serde_json::Value> {
    let now = Utc::now();

    // Count attempts in last 3 minutes
    let recent_attempts = sqlx::query!(
        "SELECT COUNT(*) as count FROM otp_attempts WHERE phone = $1 AND attempted_at > NOW() - INTERVAL '3 minutes'",
        payload.phone
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    if recent_attempts.count.unwrap_or(0) >= 3 {
        return Json(json!({
            "status": "error",
            "message": "Too many OTP requests. Try again after 3 minutes."
        }));
    }

    // Count failed attempts in last 5 minutes
    let failed_attempts = sqlx::query!(
        "SELECT COUNT(*) as count FROM otp_attempts WHERE phone = $1 AND was_success = false AND attempted_at > NOW() - INTERVAL '5 minutes'",
        payload.phone
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    if failed_attempts.count.unwrap_or(0) >= 5 {
        return Json(json!({
            "status": "error",
            "message": "Too many failed attempts. Try again after 10 minutes."
        }));
    }

    match send_otp_via_twilio_verify(&payload.phone).await {
        Ok(_) => Json(json!({ "status": "otp_sent" })),
        Err(e) => Json(json!({ "status": "error", "message": e })),
    }
}
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(input): Json<OTPInput>,
) -> Json<serde_json::Value> {
    match verify_otp_via_twilio(&input.phone, &input.otp).await {
        Ok(true) => {
            // Check if user exists
            let user_opt = sqlx::query!("SELECT id FROM users WHERE phone = $1", input.phone)
                .fetch_optional(&state.db)
                .await
                .unwrap();

            // If user exists, use ID; else insert new user and return ID
            let (user_id, is_new_user) = if let Some(user) = user_opt {
                (user.id, false)
            } else {
                let new_user = sqlx::query!(
                    "INSERT INTO users (phone) VALUES ($1) RETURNING id",
                    input.phone
                )
                .fetch_one(&state.db)
                .await
                .unwrap();
                (new_user.id, true)
            };

            // Generate secure access + refresh tokens
            let tokens = create_token_pair(user_id, &state.db).await.unwrap();

            Json(json!({
                "status": "login_success",
                "signup_required": is_new_user,
                "tokens": tokens
            }))
        }

        Ok(false) => Json(json!({
            "status": "error",
            "message": "Invalid or expired OTP"
        })),

        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })),
    }
}

