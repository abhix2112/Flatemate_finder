use axum::{
    extract::{FromRequestParts},
    http::{request::Parts, StatusCode},
};
use async_trait::async_trait;
use sqlx::PgPool;
use crate::utils::state::AppState;

pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let db = &state.db;

        let token = parts.headers.get("X-Flatmate-Token")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing token header"))?;

        let row = sqlx::query!("SELECT user_id, expires_at FROM user_tokens WHERE access_token = $1", token)
            .fetch_optional(db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error"))?;

        if let Some(row) = row {
            let now = chrono::Utc::now().naive_utc();
            if now > row.expires_at {
                return Err((StatusCode::UNAUTHORIZED, "Token expired"));
            }

            Ok(AuthenticatedUser { user_id: row.user_id })
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid token"))
        }
    }
}
