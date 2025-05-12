use axum::{routing::{get, patch, post}, serve, Router};
use tower::ServiceBuilder;
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

mod models;
mod utils;
mod routes;
mod middleware;
mod handlers;

use tower_http::services::ServeDir;

use crate::routes::{
    otp::{send_otp, verify_otp},
    user::signup,
    auth::refresh_token,
    updateprofile::update_profile,
    listing::{
        create_listing,
        get_listings,
        get_listing_by_id,
        update_listing,
        get_my_listings,
        delist_listing,
    },
 
    requests::{request_listing, get_requests_received, accept_request, reject_request},
    media::{upload_listing_media, get_listing_media},
};
use crate::handlers::auth::complete_profile;
use crate::handlers::admin_property::{get_pending_listings, approve_listing, reject_listing};
use crate::utils::state::AppState;
use middleware::logger::LoggerLayer;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&db_url).await.unwrap();
    let app_state = AppState { db: pool.clone() };

    let app = Router::new()
        .route("/send-otp", post(send_otp))
        .route("/verify-otp", post(verify_otp))
        .route("/signup", post(signup))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/complete_profile", post(complete_profile))
        .route("/update_profile", patch(update_profile))
        .route("/create_listing", post(create_listing))
        .route("/get_listings", get(get_listings))
        .route("/get_listing_by_id/:id", get(get_listing_by_id))
        .route("/update_listing/:id", patch(update_listing))
        .route("/delist_listing/:id", post(delist_listing))
        .route("/get_my_listings", get(get_my_listings))
        .route("/get_pending_listings", get(get_pending_listings))
        .route("/approve_listing/:id", post(approve_listing))
        .route("/reject_listing/:id", post(reject_listing))
        .route("/request_listing/:id", post(request_listing))
        .route("/get_requests_received", get(get_requests_received))
        .route("/accept_request/:id", post(accept_request))
        .route("/reject_request/:id", post(reject_request))
        .route("/listing/:id/media", post(upload_listing_media).get(get_listing_media))
        .nest_service("/uploads", ServeDir::new("./uploads"))
        .layer(ServiceBuilder::new().layer(LoggerLayer).into_inner())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    serve(listener, app).await.unwrap();
}
