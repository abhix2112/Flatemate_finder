use axum::{Json, extract::{Path, State}, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use crate::utils::state::AppState;
use axum::http::StatusCode;

#[derive(Deserialize)]
pub struct CreateListing {
    pub title: String,
    pub description: String,
    pub price: i32,
    pub location: String,
}

#[derive(Serialize, FromRow)]
pub struct Listing {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub price: i32,
    pub location: String,
    pub owner_id: Uuid,
}

pub async fn create_listing(
    State(state): State<AppState>,
    Json(payload): Json<CreateListing>,
) -> Result<Json<Listing>, StatusCode> {
    let rec = sqlx::query_as::<_, Listing>(
        "INSERT INTO listings (id, title, description, price, location, owner_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
    )
    .bind(Uuid::new_v4())
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.price)
    .bind(&payload.location)
    .bind(Uuid::new_v4()) // placeholder for owner_id
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rec))
}

pub async fn get_listings(State(state): State<AppState>) -> Result<Json<Vec<Listing>>, StatusCode> {
    let listings = sqlx::query_as::<_, Listing>("SELECT * FROM listings")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(listings))
}

pub async fn get_listing_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Listing>, StatusCode> {
    let listing = sqlx::query_as::<_, Listing>("SELECT * FROM listings WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(listing))
}