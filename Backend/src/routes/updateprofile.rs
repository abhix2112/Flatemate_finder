use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::middleware::auth::AuthenticatedUser;
use crate::utils::state::AppState;

#[derive(Deserialize)]
pub struct UpdateProfileInput {
    pub full_name: Option<String>,
    pub dob: Option<chrono::NaiveDate>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub city: Option<String>,
    pub preferences: Option<Value>,
}

pub async fn update_profile(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Json(input): Json<UpdateProfileInput>,
) -> Json<serde_json::Value> {
    // ✅ Role check: Only allow fm_internal users
    let role_check = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM user_roles ur
            JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = $1 AND r.name = 'fm_internal'
        )
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(false));

    if !role_check.unwrap_or(false) {
        return Json(json!({
            "status": "error",
            "message": "Unauthorized: Only verified users (fm_internal) can update their profile"
        }));
    }

    // Count updates in last 24 hrs
    let update_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM user_profiles
        WHERE user_id = $1 AND last_updated_at::date = CURRENT_DATE
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0));

    if update_count.unwrap_or_default() >= 2 {
        return Json(json!({
            "status": "error",
            "message": "Profile update limit reached for today (Max 2 updates)"
        }));
    }

    // Prevent updates to restricted fields
    if input.full_name.is_some() || input.dob.is_some() || input.email.is_some() {
        return Json(json!({
            "status": "error",
            "message": "Full name, DOB, and email cannot be updated"
        }));
    }

    // Safe update of allowed fields
    let result = sqlx::query!(
        r#"
        UPDATE user_profiles
        SET gender = COALESCE($2, gender),
            city = COALESCE($3, city),
            preferences = COALESCE($4, preferences),
            last_updated_at = NOW()
        WHERE user_id = $1
        "#,
        user_id,
        input.gender,
        input.city,
        input.preferences
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(json!({ "status": "success", "message": "Profile updated successfully" })),
        Err(e) => {
            println!("Update error: {:?}", e);
            Json(json!({ "status": "error", "message": "Profile update failed" }))
        }
    }
}
