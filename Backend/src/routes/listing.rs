use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{FromRow, Postgres, QueryBuilder};
use crate::{utils::state::AppState, middleware::auth::AuthenticatedUser};
use crate::utils::geocoding::reverse_geocode;

#[derive(Deserialize)]

pub struct CreateListing {
    pub title: String,
    pub description: String,
    pub price: i32,
    pub location: String,
    pub latitude: f64,
    pub longitude: f64,
    pub preference: Option<serde_json::Value>,
}



#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Listing {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub price: i32,
    pub location: String,
    pub latitude: f64,
    pub longitude: f64,
    pub formatted_address: String,
    pub user_id: Uuid,
    pub preference: serde_json::Value,
    pub status: String,
}




#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_km: Option<f64>,
    pub location: Option<String>,
    pub min_price: Option<i32>,
    pub max_price: Option<i32>,
}


pub async fn create_listing(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Json(payload): Json<CreateListing>,
) -> Result<Json<Listing>, StatusCode> {
    // Only internal users (admin/moderators) can create listings directly
    if !roles.contains(&"fm_internal".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let formatted_address = reverse_geocode(payload.latitude, payload.longitude)
        .await
        .unwrap_or_else(|| "Unknown Address".to_string());

    let listing_id = Uuid::new_v4();
    let preference = payload
        .preference
        .unwrap_or_else(|| serde_json::json!({}));

    // Start transaction (in case we want rollback on failure later)
    let mut tx = state.db.begin().await.map_err(|e| {
        eprintln!("Transaction begin error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Step 1: Insert the new listing
    let insert_query = r#"
        INSERT INTO listings (
            id, title, description, price, location,
            latitude, longitude, formatted_address,
            user_id, preference, status
        )
        VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8,
            $9, $10, 'draft'
        )
        RETURNING id, title, description, price, location,
                  latitude, longitude, formatted_address,
                  user_id, preference, status
    "#;

    let listing = sqlx::query_as::<_, Listing>(insert_query)
        .bind(listing_id)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(payload.price)
        .bind(&payload.location)
        .bind(payload.latitude)
        .bind(payload.longitude)
        .bind(&formatted_address)
        .bind(user_id)
        .bind(preference)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("DB insert error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if let Some(role) = sqlx::query!("SELECT id FROM roles WHERE name = 'fm_lister'")
        .fetch_optional(&state.db)
        .await
        .unwrap()
    {
        let _ = sqlx::query!(
            "INSERT INTO user_has_roles (user_id, role_id) VALUES ($1, $2)",
            user_id,
            role.id
        )
        .execute(&state.db)
        .await;
    }

    Ok(Json(listing))
}


pub async fn get_listings(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<Listing>>, StatusCode> {

    if !roles.contains(&"fm_internal".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }
    let lat = params.lat.unwrap_or(0.0);
    let lon = params.lon.unwrap_or(0.0);
    let radius_km = params.radius_km.unwrap_or(20.0); // Default to 20 km radius

    let min_price = params.min_price.unwrap_or(0);
    let max_price = params.max_price.unwrap_or(100_00_00_000); // ₹100 Cr cap

    let mut qb = QueryBuilder::<Postgres>::new(
        r#"
        SELECT * FROM (
            SELECT *, 
                6371 * acos(
                    cos(radians("#,
    );
    qb.push_bind(lat)
        .push(r#")) * cos(radians(latitude)) * cos(radians(longitude) - radians("#)
        .push_bind(lon)
        .push(r#")) + sin(radians("#)
        .push_bind(lat)
        .push(r#")) * sin(radians(latitude))
                ) AS distance
            FROM listings
            WHERE status = 'live'
        ) AS subquery
        WHERE distance <= "#);
    qb.push_bind(radius_km);

    if let Some(location) = &params.location {
        qb.push(" AND location ILIKE ");
        qb.push_bind(format!("%{}%", location));
    }

    qb.push(" AND price >= ");
    qb.push_bind(min_price);
    qb.push(" AND price <= ");
    qb.push_bind(max_price);

    qb.push(" ORDER BY distance ASC LIMIT 100");

    let query = qb.build_query_as::<Listing>();

    let listings = query
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("DB error in get_listings: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(listings))
}


pub async fn get_listing_by_id(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Listing>, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let listing = sqlx::query_as::<_, Listing>(
        "SELECT * FROM listings WHERE id = $1 AND status = 'live'"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        eprintln!("Error fetching listing: {:?}", err);
        StatusCode::NOT_FOUND
    })?;

    Ok(Json(listing))
}

pub async fn get_my_listings(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
) -> Result<Json<Vec<Listing>>, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let listings = sqlx::query_as::<_, Listing>(
        "SELECT * FROM listings WHERE owner_id = $1"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(listings))
}

pub async fn update_listing(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateListing>
) -> Result<Json<Listing>, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let formatted_address = reverse_geocode(payload.latitude, payload.longitude)
        .await
        .unwrap_or("Unknown Address".to_string());

    let listing = sqlx::query_as::<_, Listing>(
        "UPDATE listings 
         SET title = $1, description = $2, price = $3, location = $4, latitude = $5, longitude = $6, formatted_address = $7, preference = $8
         WHERE id = $9 AND owner_id = $10 RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.price)
    .bind(&payload.location)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(formatted_address)
    .bind(payload.preference.unwrap_or_else(|| serde_json::json!({})))
    .bind(id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(listing))
}

pub async fn delist_listing(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Path(id): Path<Uuid>
) -> Result<StatusCode, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query!(
        "UPDATE listings SET status = 'delist' WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
