use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::{AppState, middleware::auth::AuthenticatedUser};
use uuid::Uuid;
use serde::Serialize;

pub async fn request_listing(
    Path(listing_id): Path<Uuid>,
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
) -> Result<StatusCode, StatusCode> {
    // Only seekers can request a listing
    if !roles.contains(&"fm_internal".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM listing_requests WHERE requester_id = $1 AND listing_id = $2)",
        user_id,
        listing_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if exists.unwrap_or(false) {
        return Err(StatusCode::CONFLICT);
    }

    sqlx::query!(
        "INSERT INTO listing_requests (listing_id, requester_id) VALUES ($1, $2)",
        listing_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

#[derive(Serialize)]
pub struct RequestView {
    request_id: Uuid,
    requester_id: Uuid,
    listing_id: Uuid,
    status: String,
    created_at: chrono::NaiveDateTime,
}

pub async fn get_requests_received(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
) -> Result<Json<Vec<RequestView>>, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let rows = sqlx::query!(
        r#"
        SELECT r.id as request_id, r.requester_id, r.listing_id, r.status, r.created_at
        FROM listing_requests r
        JOIN listings l ON l.id = r.listing_id
        WHERE l.created_by = $1
        ORDER BY r.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = rows
        .into_iter()
        .map(|row| RequestView {
            request_id: row.request_id,
            requester_id: row.requester_id.unwrap(),
            listing_id: row.listing_id.unwrap(),
            status: row.status.unwrap(),
            created_at: row.created_at.unwrap(),
        })
        .collect();

    Ok(Json(response))
}

pub async fn accept_request(
    Path(request_id): Path<Uuid>,
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
) -> Result<StatusCode, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let listing = sqlx::query!(
        "SELECT l.id FROM listing_requests r JOIN listings l ON r.listing_id = l.id WHERE r.id = $1 AND l.created_by = $2",
        request_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let listing_id = match listing {
        Some(l) => l.id,
        None => return Err(StatusCode::FORBIDDEN),
    };

    let mut tx = state.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query!(
        "UPDATE listing_requests SET status = 'accepted' WHERE id = $1",
        request_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query!(
        "UPDATE listing_requests SET status = 'rejected' WHERE listing_id = $1 AND id != $2",
        listing_id,
        request_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query!(
        "UPDATE listings SET status = 'booked' WHERE id = $1",
        listing_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn reject_request(
    Path(request_id): Path<Uuid>,
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
) -> Result<StatusCode, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let listing = sqlx::query!(
        "SELECT l.id FROM listing_requests r JOIN listings l ON r.listing_id = l.id WHERE r.id = $1 AND l.created_by = $2",
        request_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if listing.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query!(
        "UPDATE listing_requests SET status = 'rejected' WHERE id = $1",
        request_id
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
