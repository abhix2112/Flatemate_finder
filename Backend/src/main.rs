use axum::{Router, routing::post, serve};
use tower_cookies::CookieManagerLayer;
use tower::ServiceBuilder;
use sqlx::postgres::PgPoolOptions;
use crate::routes::auth::refresh_token;

use dotenvy::dotenv;
use std::env;

mod models;
mod utils;
mod routes;
mod middleware;

use crate::routes::{otp::send_otp, otp::verify_otp, user::signup};
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
        .layer(ServiceBuilder::new().layer(LoggerLayer).into_inner())
        // .layer(CookieManagerLayer::new())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");
    serve(listener, app).await.unwrap();
}