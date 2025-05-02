use axum::{extract::{Path, State}, Json};
use crate::utils::state::AppState;
use uuid::Uuid;
use axum::http::StatusCode;

#[derive(Serialize, Deserialize)]
pub struct RequestAction {
    pub status: String,
}

pub async fn send_request(
    State(state): State<AppState>,
    Path(listing_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("INSERT INTO requests (id, listing_id, user_id, status) VALUES ($1, $2, $3, 'pending')")
        .bind(Uuid::new_v4())
        .bind(listing_id)
        .bind(Uuid::new_v4()) // placeholder user_id
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

pub async fn view_requests(State(state): State<AppState>) -> Result<Json<Vec<(Uuid, String)>>, StatusCode> {
    let requests = sqlx::query_as::<_, (Uuid, String)>("SELECT id, status FROM requests WHERE owner_id = $1")
        .bind(Uuid::new_v4())
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(requests))
}

pub async fn approve_request(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE requests SET status = 'approved' WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn reject_request(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE requests SET status = 'rejected' WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}