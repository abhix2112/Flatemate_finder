use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use sqlx::FromRow;
use crate::{
    middleware::auth::AuthenticatedUser,
    utils::state::AppState,
    routes::listing::Listing,
};

#[derive(FromRow)]
struct ListingStatusOnly {
    pub status: String,
}

// Middleware check
fn is_admin(roles: &Vec<String>) -> bool {
    roles.contains(&"admin".to_string()) || roles.contains(&"super_admin".to_string())
}

// 🔒 GET all listings pending approval
pub async fn get_pending_listings(
    State(state): State<AppState>,
    AuthenticatedUser { roles, .. }: AuthenticatedUser,
) -> Result<Json<Vec<Listing>>, StatusCode> {
    if !is_admin(&roles) {
        return Err(StatusCode::FORBIDDEN);
    }

    let listings = sqlx::query_as::<_, Listing>(
        "SELECT * FROM listings WHERE status = 'under_review'"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(listings))
}

// 🔒 Approve a listing (status → live)
pub async fn approve_listing(
    State(state): State<AppState>,
    AuthenticatedUser { roles, .. }: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    if !is_admin(&roles) {
        return Err(StatusCode::FORBIDDEN);
    }

    let result = sqlx::query!(
        "UPDATE listings SET status = 'live' WHERE id = $1 AND status = 'under_review'",
        id
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

// 🔒 Reject a listing (status → draft)
pub async fn reject_listing(
    State(state): State<AppState>,
    AuthenticatedUser { roles, .. }: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    if !is_admin(&roles) {
        return Err(StatusCode::FORBIDDEN);
    }

    let result = sqlx::query!(
        "UPDATE listings SET status = 'draft' WHERE id = $1 AND status = 'under_review'",
        id
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}
