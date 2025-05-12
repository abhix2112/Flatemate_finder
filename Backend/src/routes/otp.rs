use axum::{Json, extract::State};
use serde_json::json;
use chrono::{Utc};
use crate::utils::state::AppState;
use crate::utils::twilio::{send_otp_via_twilio_verify, verify_otp_via_twilio};
use crate::models::otp::{OTPInput, Sendotp};
use crate::utils::token::create_token_pair;
use uuid::Uuid;

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
            let mut is_new_user = false;

            // 1. Check if user exists
            let user_opt = sqlx::query!("SELECT id FROM users WHERE phone = $1", input.phone)
                .fetch_optional(&state.db)
                .await
                .unwrap();

            let user_id = if let Some(user) = user_opt {
                user.id
            } else {
                let new_user = sqlx::query!(
                    "INSERT INTO users (phone) VALUES ($1) RETURNING id",
                    input.phone
                )
                .fetch_one(&state.db)
                .await
                .unwrap();
                is_new_user = true;
                new_user.id
            };

            // 2. If new user, assign default role `fm_external`
            if is_new_user {
                // Fetch role ID of 'fm_external'
                if let Some(role) = sqlx::query!("SELECT id FROM roles WHERE name = 'fm_external'")
                    .fetch_optional(&state.db)
                    .await
                    .unwrap()
                {
                    let _ = sqlx::query!(
                        "INSERT INTO user_has_roles (user_id, role_id) VALUES ($1, $2)",
                        user_id,
                        role.id
                    )
                    .execute(&state.db)
                    .await;
                }
            }

            // 3. Fetch all user roles
            let roles = sqlx::query!(
                r#"
                SELECT r.name FROM roles r
                JOIN user_has_roles ur ON ur.role_id = r.id
                WHERE ur.user_id = $1
                "#,
                user_id
            )
            .fetch_all(&state.db)
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.name)
            .collect::<Vec<String>>();

            // 4. Save OTP success for attempt tracking
            let _ = sqlx::query!(
                "INSERT INTO otp_attempts (phone, was_success, attempted_at) VALUES ($1, true, NOW())",
                input.phone
            )
            .execute(&state.db)
            .await;

            // 5. Create tokens
            let tokens = create_token_pair(user_id, &state.db).await.unwrap();

            Json(json!({
                "status": "login_success",
                "signup_required": is_new_user,
                "roles": roles,
                "tokens": tokens
            }))
        }

        Ok(false) => {
            // Save failed attempt
            let _ = sqlx::query!(
                "INSERT INTO otp_attempts (phone, was_success, attempted_at) VALUES ($1, false, NOW())",
                input.phone
            )
            .execute(&state.db)
            .await;

            Json(json!({
                "status": "error",
                "message": "Invalid or expired OTP"
            }))
        }

        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })),
    }
}

