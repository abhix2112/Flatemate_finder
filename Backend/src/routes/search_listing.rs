use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use crate::{AppState, middleware::auth::AuthenticatedUser};
use crate::routes::listing::Listing;

#[derive(Deserialize, Debug)]
pub struct SearchParams {
    pub location: Option<String>,
    pub min_price: Option<i32>,
    pub max_price: Option<i32>,
    pub gender_preference: Option<String>,
    pub sort_by: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn search_listing(
    State(state): State<AppState>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Listing>>, StatusCode> {
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(10).min(50);

    let mut sql = String::from("SELECT * FROM listings WHERE status = 'live'");
    let mut binds: Vec<Box<dyn Fn(sqlx::query::QueryAs<'_, sqlx::Postgres, Listing, sqlx::postgres::PgArguments>) -> sqlx::query::QueryAs<'_, sqlx::Postgres, Listing, sqlx::postgres::PgArguments> + Send + Sync>> = vec![];

    if let Some(location) = &params.location {
        sql += " AND location ILIKE $1";
        let loc = format!("%{}%", location.clone());
        binds.push(Box::new(move |q| q.bind(loc.clone())));
    }

    if let Some(min_price) = params.min_price {
        sql += " AND price >= $2";
        binds.push(Box::new(move |q| q.bind(min_price)));
    }

    if let Some(max_price) = params.max_price {
        sql += " AND price <= $3";
        binds.push(Box::new(move |q| q.bind(max_price)));
    }

    if let Some(gender) = &params.gender_preference {
        sql += " AND preference ->> 'gender' = $4";
        let gender = gender.clone();
        binds.push(Box::new(move |q| q.bind(gender.clone())));
    }

    let sort_by_clause = match params.sort_by.as_deref() {
        Some("price_low_to_high") => " ORDER BY price ASC",
        Some("price_high_to_low") => " ORDER BY price DESC",
        Some("best_match") => " ORDER BY location, price",
        _ => " ORDER BY created_at DESC",
    };

    sql += sort_by_clause;
    sql += " LIMIT $5 OFFSET $6";

    // Build final query
    let mut query = sqlx::query_as::<_, Listing>(&sql);
    for bind_fn in binds {
        query = bind_fn(query);
    }
    query = query.bind(limit).bind(offset);

    let listings = query
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("DB Error in search_listings: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(listings))
}
