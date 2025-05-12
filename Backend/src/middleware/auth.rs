use axum::{
    extract::{FromRequestParts, FromRef},
    http::{request::Parts, StatusCode},
};
use async_trait::async_trait;
use sqlx::PgPool;
use crate::utils::state::AppState;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub roles: Vec<String>,
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

        // 1. Extract Bearer token
        let token = parts.headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or((StatusCode::UNAUTHORIZED, "Missing or malformed Authorization header"))?;

        // 2. Validate token from DB
        let row = sqlx::query!(
            "SELECT user_id, expires_at FROM user_tokens WHERE access_token = $1",
            token
        )
        .fetch_optional(db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error"))?;

        let user_token = match row {
            Some(row) if chrono::Utc::now().naive_utc() <= row.expires_at => row,
            Some(_) => return Err((StatusCode::UNAUTHORIZED, "Token expired")),
            None => return Err((StatusCode::UNAUTHORIZED, "Invalid token")),
        };

        // 3. Fetch user roles
        let roles_rows = sqlx::query!(
            r#"
            SELECT r.name
            FROM roles r
            INNER JOIN user_has_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = $1
            "#,
            user_token.user_id
        )
        .fetch_all(db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch roles"))?;

        let roles = roles_rows.into_iter().map(|r| r.name).collect::<Vec<_>>();

        Ok(AuthenticatedUser {
            user_id: user_token.user_id,
            roles,
        })
    }
}
