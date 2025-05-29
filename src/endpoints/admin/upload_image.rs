use crate::middleware::auth::auth_middleware;
use crate::models::AppState;
use crate::utils::get_error;
use axum::{
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use serde_json::json;
use std::sync::Arc;
use std::{fs::create_dir_all, path::Path as FilePath};

#[route(post, "/admin/images/upload/:image_name", auth_middleware)]
pub async fn upload_image_handler(
    State(_state): State<Arc<AppState>>,
    Extension(_sub): Extension<String>, // Example if sub is needed for authorization
    Path(image_name): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let images_folder = "./images";
    if !FilePath::new(images_folder).exists() {
        if let Err(e) = create_dir_all(images_folder) {
            return get_error(format!("Failed to create images folder: {}", e));
        }
    }

    while let Some(field) = multipart.next_field().await.unwrap_or_else(|_| None) {
        if let Some(filename) = field.file_name() {
            if filename.ends_with(".webp") {
                let filepath = format!("{}/{}.webp", images_folder, image_name);
                let data = match field.bytes().await {
                    Ok(data) => data,
                    Err(e) => {
                        return get_error(format!("Failed to read file data: {}", e));
                    }
                };

                if let Err(e) = tokio::fs::write(&filepath, data).await {
                    return get_error(format!("Failed to write file: {}", e));
                }

                return (
                    StatusCode::OK,
                    Json(json!({"message": "File uploaded successfully"})),
                )
                    .into_response();
            } else {
                return get_error("Only .webp files are allowed".to_string());
            }
        }
    }

    get_error("No valid file provided".to_string())
}
