use axum::{Json, extract::State};
use serde_json::json;
use crate::utils::state::AppState;
use crate::utils::twilio::{send_otp_via_twilio_verify, verify_otp_via_twilio};
use crate::models::{OTPInput, Sendotp};

pub async fn send_otp(
    Json(payload): Json<Sendotp>,
) -> Json<serde_json::Value> {
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
            let existing = sqlx::query!("SELECT id FROM users WHERE phone = $1", input.phone)
                .fetch_optional(&state.db)
                .await
                .unwrap();

            if existing.is_some() {
                Json(json!({ "status": "login_success", "message": "User exists" }))
            } else {
                Json(json!({ "status": "signup_required", "message": "New user. Proceed to signup." }))
            }
        }
        Ok(false) => Json(json!({ "status": "error", "message": "Invalid or expired OTP" })),
        Err(e) => Json(json!({ "status": "error", "message": e })),
    }
}