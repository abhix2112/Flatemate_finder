use axum::{Router, routing::post};
use tower_cookies::CookieManagerLayer;
use tower::Layer;
use sqlx::postgres::PgPoolOptions;

mod models;
mod utils;
mod routes;
mod middleware;

use crate::routes::{otp::send_otp, otp::verify_otp, jwt::refresh_token, user::signup};
use crate::utils::state::AppState;
use middleware::logger::LoggerLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&db_url).await?;

    let app_state = AppState { db: pool.clone() };

    // Add routes first
    let mut app = Router::new()
        .route("/send-otp", post(send_otp))
        .route("/verify-otp", post(verify_otp))
        .route("/signup", post(signup))
        .route("/refresh", post(refresh_token))
        // Add logger middleware
        .layer(LoggerLayer)
        .with_state(app_state);
        
    // Add cookie middleware


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    
    Ok(axum::serve(listener, app).await?)
}