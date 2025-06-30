use axum::{
    body::StreamBody,
    extract::Path,
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
};
use axum_auto_routes::route;
use mongodb::bson::doc;
use tokio_util::io::ReaderStream;

#[route(get, "/images/:image_name")]
pub async fn handler(Path(image_name): Path<String>) -> impl IntoResponse {
    let images_folder = "./images";

    let filepath = format!("{}/{}.webp", images_folder, image_name);

    let file = match tokio::fs::File::open(filepath).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    let headers = AppendHeaders([(header::CONTENT_TYPE, "image/webp")]);
    let body = StreamBody::new(ReaderStream::new(file));

    Ok((headers, body))
}
