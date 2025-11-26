use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
};
use collie::scraper::screenscraper::ScreenScraperConfig;
use collie::scraper::thegamesdb::TheGamesDBConfig;
use collie::{ProgressUpdate, scraper::gamefaqs::GameFAQsConfig};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use tracing::debug;

use crate::server::state::{AppState, ScrapingState, load_state, save_state};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataBackendConfigs {
    pub screenscraper: Option<ScreenScraperConfig>,
    pub thegamesdb: Option<TheGamesDBConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideBackendConfigs {
    pub gamefaqs: Option<GameFAQsConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeRequest {
    pub roms_path: String,
    pub box_art_width: u32,
    pub skip_cache: bool,
    pub metadata_backends: MetadataBackendConfigs,
    pub guide_backends: GuideBackendConfigs,
}

#[derive(Debug, Serialize)]
pub struct ScrapeResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSettingsRequest {
    pub roms_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStateRequest {
    pub roms_path: String,
}

#[derive(Debug, Serialize)]
pub struct SaveSettingsResponse {
    pub success: bool,
    pub message: String,
    pub state: ScrapingState,
}

pub async fn save_settings(
    State(state): State<AppState>,
    Json(request): Json<SaveSettingsRequest>,
) -> Result<Json<SaveSettingsResponse>, StatusCode> {
    let scraping = state.scraping.lock().await;

    if *scraping {
        return Ok(Json(SaveSettingsResponse {
            success: false,
            message: "Cannot change settings while scraping is in progress".to_string(),
            state: ScrapingState::default(),
        }));
    }
    drop(scraping);

    // Update roms_path
    let roms_path = std::path::PathBuf::from(&request.roms_path);
    *state.roms_path.lock().unwrap() = roms_path.clone();
    tracing::info!("Updated ROMs path to: {}", roms_path.display());

    // Load state from the new ROMs path
    let loaded_state = load_state(&roms_path);

    // Update the app's scraping state
    {
        let mut scraping_state = state.scraping_state.lock().await;
        *scraping_state = loaded_state.clone();
    }

    Ok(Json(SaveSettingsResponse {
        success: true,
        message: format!("Settings saved. Loaded state from {}", roms_path.display()),
        state: loaded_state,
    }))
}

pub async fn get_state(
    State(state): State<AppState>,
    Json(request): Json<GetStateRequest>,
) -> Json<ScrapingState> {
    let roms_path = std::path::PathBuf::from(&request.roms_path);

    // Update the app's roms_path so future API requests use this path
    *state.roms_path.lock().unwrap() = roms_path.clone();

    // Check if we're currently scraping
    let scraping = *state.scraping.lock().await;

    if scraping {
        // Return the live in-memory state if actively scraping
        let scraping_state = state.scraping_state.lock().await;
        Json(scraping_state.clone())
    } else {
        // Load state from disk for the specified roms_path
        let loaded_state = load_state(&roms_path);
        // Also update the in-memory scraping state
        *state.scraping_state.lock().await = loaded_state.clone();
        Json(loaded_state)
    }
}

pub async fn progress_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.progress_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .map(|msg: Result<String, _>| Ok(Event::default().data(msg.unwrap_or_default())));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(1))
            .text("keep-alive"),
    )
}

pub async fn start_scraping(
    State(state): State<AppState>,
    Json(request): Json<ScrapeRequest>,
) -> Result<Json<ScrapeResponse>, StatusCode> {
    let mut scraping = state.scraping.lock().await;

    if *scraping {
        return Ok(Json(ScrapeResponse {
            success: false,
            message: "Scraping already in progress".to_string(),
        }));
    }

    *scraping = true;
    drop(scraping);

    // Update roms_path to use the path from the UI request
    let roms_path = std::path::PathBuf::from(&request.roms_path);
    *state.roms_path.lock().unwrap() = roms_path.clone();
    tracing::info!("Updated ROMs path to: {}", roms_path.display());

    // Clear cache if skip_cache is enabled
    if request.skip_cache {
        tracing::info!("Clearing cache and stored data...");
        let collie_dir = roms_path.join(".collie");

        // Clear the failure markers cache
        use collie::cache::ScrapeCache;
        let cache = ScrapeCache::new(&roms_path);
        if let Err(e) = cache.clear_all() {
            tracing::warn!("Failed to clear cache: {}", e);
        }

        // Clear stored game data
        let games_dir = collie_dir.join("games");
        if games_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&games_dir) {
                tracing::warn!("Failed to clear games directory: {}", e);
            }
        }

        // Clear games index
        let games_index = collie_dir.join("games.txt");
        if games_index.exists() {
            if let Err(e) = std::fs::remove_file(&games_index) {
                tracing::warn!("Failed to clear games index: {}", e);
            }
        }

        // Clear scraping state
        let state_file = collie_dir.join("state.json");
        if state_file.exists() {
            if let Err(e) = std::fs::remove_file(&state_file) {
                tracing::warn!("Failed to clear state file: {}", e);
            }
        }

        // Clear crawled file
        let crawled_file = collie_dir.join("crawled");
        if crawled_file.exists() {
            if let Err(e) = std::fs::remove_file(&crawled_file) {
                tracing::warn!("Failed to clear crawled file: {}", e);
            }
        }

        tracing::info!("Cache and stored data cleared");
    }

    // Create cancellation token
    let cancel_token = tokio_util::sync::CancellationToken::new();
    *state.cancel_token.lock().await = Some(cancel_token.clone());

    // Reset state before starting new scraping session
    {
        let mut scraping_state = state.scraping_state.lock().await;
        *scraping_state = ScrapingState {
            scraping: true,
            ..Default::default()
        };
    }

    // Spawn scraping task in background
    let progress_tx = state.progress_tx.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        let result = run_scraping(
            request,
            progress_tx.clone(),
            state_clone.clone(),
            cancel_token,
        )
        .await;
        let mut scraping = state_clone.scraping.lock().await;
        *scraping = false;
        *state_clone.cancel_token.lock().await = None;

        // Update scraping state
        {
            let mut scraping_state = state_clone.scraping_state.lock().await;
            scraping_state.scraping = false;
        }

        match result {
            Ok(_) => {
                // Progress updates are already sent via the callback
            }
            Err(e) => {
                tracing::error!("Scraping failed: {}", e);
                // Send error message to UI via progress channel
                let error_msg = format!(
                    "{{\"message\": \"Scraping failed: {}\", \"completed\": 0, \"total\": 0, \"success_count\": 0, \"fail_count\": 0, \"skip_count\": 0}}",
                    e
                );
                let _ = progress_tx.send(error_msg);
            }
        }
    });

    Ok(Json(ScrapeResponse {
        success: true,
        message: "Scraping started".to_string(),
    }))
}

pub async fn stop_scraping(
    State(state): State<AppState>,
) -> Result<Json<ScrapeResponse>, StatusCode> {
    let scraping = state.scraping.lock().await;

    if !*scraping {
        return Ok(Json(ScrapeResponse {
            success: false,
            message: "No scraping in progress".to_string(),
        }));
    }
    drop(scraping);

    // Cancel the scraping task
    if let Some(token) = state.cancel_token.lock().await.as_ref() {
        token.cancel();
        tracing::info!("Scraping cancellation requested");
    }

    Ok(Json(ScrapeResponse {
        success: true,
        message: "Scraping cancelled".to_string(),
    }))
}

async fn run_scraping(
    request: ScrapeRequest,
    progress_json_tx: tokio::sync::broadcast::Sender<String>,
    app_state: AppState,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use collie::scraper::gamefaqs::GameFAQsScraper;
    use collie::scraper::screenscraper::ScreenScraper;
    use collie::scraper::thegamesdb::TheGamesDB;
    use collie::scraper::{GuidesScraper, MetadataScraper};
    use collie::{ScrapingConfig, scrape};

    let roms_path = std::path::PathBuf::from(&request.roms_path);

    let mut metadata_scrapers: Vec<Box<dyn MetadataScraper>> = Vec::new();

    if let Some(ss_config) = request.metadata_backends.screenscraper {
        let mut scraper = ScreenScraper::new().with_box_art_type(ss_config.box_art_type);

        if let (Some(username), Some(password)) = (ss_config.username, ss_config.password) {
            scraper.authenticate(&username, &password).await?;
        }

        metadata_scrapers.push(Box::new(scraper));
    }

    if let Some(tgdb_config) = request.metadata_backends.thegamesdb {
        let scraper = TheGamesDB::with_api_key(tgdb_config.api_key);
        metadata_scrapers.push(Box::new(scraper));
    }

    let mut guides_scrapers: Vec<Box<dyn GuidesScraper>> = Vec::new();

    if let Some(_gamefaqs_config) = request.guide_backends.gamefaqs {
        guides_scrapers.push(Box::new(GameFAQsScraper::new()));
    }

    if metadata_scrapers.is_empty() && guides_scrapers.is_empty() {
        return Err(
            "No scrapers configured. Please enable at least one metadata backend or guide backend."
                .into(),
        );
    }

    // Run scraping with progress updates via channel
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<ProgressUpdate>();

    // Spawn a task to handle progress updates
    let roms_path_for_state = roms_path.clone();
    tokio::spawn(async move {
        while let Some(update) = progress_rx.recv().await {
            // Update stored state
            if let Ok(mut state) = app_state.scraping_state.try_lock() {
                state.progress = update.completed;
                state.total_games = update.total;
                state.success_count = update.success_count;
                state.fail_count = update.fail_count;
                state.skip_count = update.skip_count;
                state.current_message = update.message.clone();

                // Save state to disk
                save_state(&roms_path_for_state, &state);
            }

            // Send progress update via SSE
            let json = serde_json::to_string(&update).unwrap_or_default();
            debug!("Sending progress update: {}", json);
            let _ = progress_json_tx.send(json);
        }
    });

    let skip_cache = request.skip_cache;
    let progress = tokio::spawn(async move {
        let config = ScrapingConfig {
            roms_path,
            images_folder: "Imgs".to_string(),
            guides_folder: "Guides".to_string(),
            box_art_width: Some(request.box_art_width),
            skip_cache,
        };

        scrape(
            metadata_scrapers,
            guides_scrapers,
            config,
            cancel_token,
            progress_tx,
        )
        .await
    })
    .await??;

    Ok(format!(
        "Scraping complete! Total: {}, Success: {}, Skipped: {}, Failed: {}",
        progress.total, progress.success_count, progress.skip_count, progress.fail_count
    ))
}
