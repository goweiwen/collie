use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Serialize)]
pub struct GamesResponse {
    pub games: Vec<String>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

pub async fn get_games(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Json<GamesResponse> {
    let roms_path = state.roms_path.lock().unwrap().clone();
    let mut all_games = load_scraped_index(&roms_path);
    let total = all_games.len();

    // Reverse to show newest games first (games.txt has oldest first)
    all_games.reverse();

    // Apply pagination
    let games = all_games
        .into_iter()
        .skip(params.offset)
        .take(params.limit)
        .collect();

    Json(GamesResponse {
        games,
        total,
        offset: params.offset,
        limit: params.limit,
    })
}

pub async fn get_game_by_rom_name(
    State(state): State<AppState>,
    Path(rom_name): Path<String>,
) -> Result<Json<collie::GameData>, StatusCode> {
    let roms_path = state.roms_path.lock().unwrap().clone();

    match load_game_by_rom_name(&roms_path, &rom_name) {
        Some(game_data) => Ok(Json(game_data)),
        None => {
            tracing::warn!("Game not found: {}", rom_name);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Load scraped games index from .collie/games.txt
fn load_scraped_index(roms_path: &std::path::Path) -> Vec<String> {
    let games_index_file = roms_path.join(".collie").join("games.txt");
    let mut games = Vec::new();

    if !games_index_file.exists() {
        return games;
    }

    match std::fs::read_to_string(&games_index_file) {
        Ok(content) => {
            // Read plain text lines (one ROM file path per line)
            for line in content.lines() {
                let rom_name = line.trim();
                if !rom_name.is_empty() {
                    games.push(rom_name.to_string());
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to read scraped index file: {}", e);
        }
    }

    if !games.is_empty() {
        tracing::info!("Loaded {} games from index", games.len());
    }

    games
}

/// Load a single game by ROM name by scanning all game files
fn load_game_by_rom_name(roms_path: &std::path::Path, rom_name: &str) -> Option<collie::GameData> {
    let games_dir = roms_path.join(".collie").join("games");

    if !games_dir.exists() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(&games_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(game_data) = serde_json::from_str::<collie::GameData>(&content) {
                        if game_data.rom_name == rom_name {
                            return Some(game_data);
                        }
                    }
                }
            }
        }
    }

    None
}
