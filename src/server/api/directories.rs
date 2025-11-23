use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDirectoriesRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDirectoriesResponse {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub directories: Vec<DirectoryEntry>,
}

pub async fn list_directories(
    Json(request): Json<ListDirectoriesRequest>,
) -> Result<Json<ListDirectoriesResponse>, StatusCode> {
    // Get current working directory as the security boundary
    let cwd = std::env::current_dir().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse and canonicalize the requested path
    let requested_path = if request.path.is_empty() || request.path == "." {
        cwd.clone()
    } else {
        PathBuf::from(&request.path)
    };

    let canonical_requested = requested_path
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // // Security check: ensure path is within cwd
    // if !canonical_requested.starts_with(&cwd) {
    //     tracing::warn!(
    //         "Attempted directory traversal: {} is outside {}",
    //         canonical_requested.display(),
    //         cwd.display()
    //     );
    //     return Err(StatusCode::FORBIDDEN);
    // }

    // Calculate parent path (only if still within cwd)
    let parent_path = canonical_requested.parent().and_then(|parent| {
        if parent.starts_with(&cwd) || parent == cwd {
            Some(parent.to_string_lossy().to_string())
        } else {
            None
        }
    });

    // Read directory contents
    let entries = std::fs::read_dir(&canonical_requested).map_err(|e| {
        tracing::error!("Failed to read directory: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Filter to only directories and collect
    let mut directories: Vec<DirectoryEntry> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter(|entry| {
            // Filter out hidden directories (starting with .)
            entry
                .file_name()
                .to_str()
                .map(|name| !name.starts_with('.'))
                .unwrap_or(false)
        })
        .map(|entry| DirectoryEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
        })
        .collect();

    // Sort alphabetically by name
    directories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(Json(ListDirectoriesResponse {
        current_path: canonical_requested.to_string_lossy().to_string(),
        parent_path,
        directories,
    }))
}
