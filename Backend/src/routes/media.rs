use axum::{
    extract::{Multipart, Path, State},
    response::Json,
    http::StatusCode,
};
use crate::middleware::auth::AuthenticatedUser;
use serde::Serialize;
use std::{fs::create_dir_all, io::Write, path::PathBuf};
use uuid::Uuid;

use crate::AppState;

const UPLOAD_DIR: &str = "./uploads";

#[derive(Serialize)]
pub struct MediaItem {
    pub id: Uuid,
    pub media_url: String,
    pub media_type: String,
    pub uploaded_at: chrono::NaiveDateTime,
}

/// POST /listing/:id/media
pub async fn upload_listing_media(
    Path(listing_id): Path<Uuid>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    if !roles.contains(&"fm_lister".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    create_dir_all(UPLOAD_DIR).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("file").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        // Determine media type
        let media_type = if content_type.starts_with("image/") {
            "image"
        } else if content_type.starts_with("video/") {
            "video"
        } else {
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        };

        let path = PathBuf::from(&file_name);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");

        let unique_file_name = format!("{}_{}.{}", listing_id, Uuid::new_v4(), extension);
        let file_path = format!("{}/{}", UPLOAD_DIR, unique_file_name);

        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        let mut file = std::fs::File::create(&file_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write_all(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let url = format!("/uploads/{}", unique_file_name); // exposed by Axum static route

        // Store metadata
        sqlx::query!(
            "INSERT INTO listing_media (listing_id, media_url, media_type) VALUES ($1, $2, $3)",
            listing_id,
            url,
            media_type
        )
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::CREATED)
}

/// GET /listing/:id/media
pub async fn get_listing_media(
    Path(listing_id): Path<Uuid>,
    AuthenticatedUser { user_id, roles }: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<MediaItem>>, StatusCode> {
    if !roles.contains(&"fm_internal".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let rows = sqlx::query!(
        r#"
        SELECT id, media_url, media_type, uploaded_at
        FROM listing_media
        WHERE listing_id = $1 AND is_active = true
        ORDER BY uploaded_at DESC
        "#,
        listing_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = rows
        .into_iter()
        .map(|r| MediaItem {
            id: r.id,
            media_url: r.media_url,
            media_type: r.media_type,
            uploaded_at: r.uploaded_at.unwrap_or_default(),
        })
        .collect();

    Ok(Json(result))
}
