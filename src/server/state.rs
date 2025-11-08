use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScrapingState {
    pub scraping: bool,
    pub progress: usize,
    pub total_games: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
    pub current_message: String,
}

#[derive(Clone)]
pub struct AppState {
    pub scraping: Arc<Mutex<bool>>,
    pub cancel_token: Arc<Mutex<Option<tokio_util::sync::CancellationToken>>>,
    pub progress_tx: broadcast::Sender<String>,
    pub roms_path: Arc<std::sync::Mutex<std::path::PathBuf>>,
    pub scraping_state: Arc<Mutex<ScrapingState>>,
}

/// Save scraping state to .collie/state.json
pub fn save_state(roms_path: &Path, state: &ScrapingState) {
    let collie_dir = roms_path.join(".collie");
    if let Err(e) = std::fs::create_dir_all(&collie_dir) {
        tracing::warn!("Failed to create .collie directory: {}", e);
        return;
    }

    let state_file = collie_dir.join("state.json");
    match serde_json::to_string_pretty(state) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&state_file, json) {
                tracing::warn!("Failed to save state: {}", e);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to serialize state: {}", e);
        }
    }
}

/// Load scraping state from .collie/state.json
pub fn load_state(roms_path: &Path) -> ScrapingState {
    let state_file = roms_path.join(".collie").join("state.json");

    if !state_file.exists() {
        return ScrapingState::default();
    }

    match std::fs::read_to_string(&state_file) {
        Ok(content) => {
            match serde_json::from_str::<ScrapingState>(&content) {
                Ok(mut state) => {
                    // Don't restore scraping=true on load
                    state.scraping = false;
                    tracing::info!(
                        "Loaded state: {}/{} games processed",
                        state.progress,
                        state.total_games
                    );
                    state
                }
                Err(e) => {
                    tracing::warn!("Failed to parse state file: {}", e);
                    ScrapingState::default()
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to read state file: {}", e);
            ScrapingState::default()
        }
    }
}
