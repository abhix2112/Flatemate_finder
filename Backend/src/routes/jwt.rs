use axum::response::IntoResponse;
use axum::http::StatusCode;

pub async fn refresh_token() -> impl IntoResponse {
    (StatusCode::OK, "Token refreshed")
}
