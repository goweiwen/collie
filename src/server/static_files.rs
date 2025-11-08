use axum::{
    body::Body,
    http::{StatusCode, Uri, header},
    response::{Html, IntoResponse, Response},
};

#[cfg(feature = "bundled")]
use rust_embed::RustEmbed;

#[cfg(feature = "bundled")]
#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Assets;

#[cfg(feature = "bundled")]
pub async fn static_handler(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return serve_index();
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                StatusCode::NOT_FOUND.into_response()
            } else {
                serve_index()
            }
        }
    }
}

#[cfg(feature = "bundled")]
fn serve_index() -> Response<Body> {
    match Assets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(not(feature = "bundled"))]
pub async fn static_handler(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    let frontend_dir = std::path::Path::new("frontend/dist");

    if path.is_empty() || path == "index.html" {
        return serve_index();
    }

    let file_path = frontend_dir.join(path);

    // Security check: ensure the path is within frontend_dir
    let Ok(canonical) = file_path.canonicalize() else {
        if path.contains('.') {
            return StatusCode::NOT_FOUND.into_response();
        } else {
            return serve_index();
        }
    };
    let Ok(base) = frontend_dir.canonicalize() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if !canonical.starts_with(base) {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Check if file exists and is a file
    if !file_path.is_file() {
        if path.contains('.') {
            return StatusCode::NOT_FOUND.into_response();
        } else {
            return serve_index();
        }
    }

    // Read and serve file
    match tokio::fs::read(&file_path).await {
        Ok(contents) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], contents).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(not(feature = "bundled"))]
fn serve_index() -> Response<Body> {
    let index_path = std::path::Path::new("frontend/dist/index.html");

    match std::fs::read(index_path) {
        Ok(content) => Html(content).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
