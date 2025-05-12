use axum::{Json, extract::State};
use crate::utils::state::AppState;
use crate::middleware::auth::AuthenticatedUser;
use serde::Deserialize;
use serde_json::{json, Value};
use chrono::NaiveDate;

#[derive(Deserialize)]
pub struct CompleteProfileInput {
    pub full_name: String,
    pub dob: NaiveDate,
    pub email: String,
    pub gender: Option<String>,
    pub city: Option<String>,
    pub preferences: Option<Value>, // expecting JSON from frontend
}

#[axum::debug_handler]
pub async fn complete_profile(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Json(input): Json<CompleteProfileInput>,
) -> Json<serde_json::Value> {
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return Json(json!({ "status": "error", "message": "Failed to start DB transaction" }));
        }
    };

    // ✅ Check if the user has the fm_external role
    let current_role = sqlx::query_scalar!(
        "SELECT r.name AS role_name
FROM user_has_roles u
JOIN roles r ON u.role_id = r.id
WHERE u.user_id = $1;",
        user_id
    )
    .fetch_optional(&mut *tx)
    .await;

    match current_role {
        Ok(Some(role)) if role != "fm_external" => {
            return Json(json!({
                "status": "error",
                "message": "Only users with 'fm_external' role can complete profile"
            }));
        }
        Ok(None) => {
            return Json(json!({
                "status": "error",
                "message": "User or role not found"
            }));
        }
        Err(e) => {
            println!("Role check error: {:?}", e);
            return Json(json!({ "status": "error", "message": "Failed to check user role" }));
        }
        _ => {}
    }

    // ✅ Insert into user_profiles
    if let Err(e) = sqlx::query!(
        "INSERT INTO user_profiles (user_id, full_name, dob, email, gender, city, preferences)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        user_id,
        input.full_name,
        input.dob,
        input.email,
        input.gender,
        input.city,
        input.preferences
    )
    .execute(&mut *tx)
    .await {
        println!("Profile insert error: {:?}", e);
        return Json(json!({ "status": "error", "message": "Failed to complete profile" }));
    }

    // ✅ Update is_profile_complete = true
    if let Err(e) = sqlx::query!(
        "UPDATE users SET is_profile_complete = true WHERE id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await {
        println!("User update error: {:?}", e);
        return Json(json!({ "status": "error", "message": "Profile saved but user update failed" }));
    }

    // ✅ Promote role to fm_internal
    let internal_role_id = sqlx::query_scalar!(
        "SELECT id FROM roles WHERE name = 'fm_internal'"
    )
    .fetch_one(&mut *tx)
    .await;

    match internal_role_id {
        Ok(role_id) => {
            if let Err(e) = sqlx::query!(
                "UPDATE user_has_roles SET role_id = $1 WHERE user_id = $2",
                role_id,
                user_id
            )
            .execute(&mut *tx)
            .await {
                println!("Role update error: {:?}", e);
                return Json(json!({ "status": "error", "message": "Profile updated but failed to upgrade role" }));
            }
        }
        Err(e) => {
            println!("Fetching fm_internal role_id failed: {:?}", e);
            return Json(json!({ "status": "error", "message": "Failed to fetch new role" }));
        }
    }

    if let Err(e) = tx.commit().await {
        println!("Transaction commit error: {:?}", e);
        return Json(json!({ "status": "error", "message": "Profile update failed at commit" }));
    }

    Json(json!({ "status": "success", "message": "Profile completed and role upgraded" }))
}
