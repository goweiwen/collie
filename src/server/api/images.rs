use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::server::state::AppState;

pub async fn serve_image(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Response<Body> {
    if path.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // Get the current roms_path
    let roms_path = state.roms_path.lock().unwrap().clone();

    // Build full path to image file
    let image_path = roms_path.join(&path);

    // Security check: ensure the path is within roms_path
    let Ok(canonical) = image_path.canonicalize() else {
        tracing::warn!("Failed to canonicalize image path: {}", path);
        return StatusCode::FORBIDDEN.into_response();
    };
    let Ok(base) = roms_path.canonicalize() else {
        tracing::warn!("Failed to canonicalize roms path");
        return StatusCode::FORBIDDEN.into_response();
    };
    if !canonical.starts_with(base) {
        tracing::warn!("Forbidden path access attempt: {}", path);
        return StatusCode::FORBIDDEN.into_response();
    }

    // Check if file exists and is a file
    if !image_path.is_file() {
        tracing::debug!(
            "Image not found: {} (base: {})",
            image_path.display(),
            roms_path.display()
        );
        return StatusCode::NOT_FOUND.into_response();
    }

    // Read file
    match tokio::fs::read(&image_path).await {
        Ok(contents) => {
            let mime = mime_guess::from_path(&image_path).first_or_octet_stream();
            tracing::debug!("Serving image: {} ({})", image_path.display(), mime);
            ([(header::CONTENT_TYPE, mime.as_ref())], contents).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to read image {}: {}", image_path.display(), e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
