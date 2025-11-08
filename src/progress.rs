use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrapeStatus {
    Pending,
    Searching,
    Success,
    Failed,
    Skipped,
}

impl ScrapeStatus {
    pub fn merge(self, other: ScrapeStatus) -> ScrapeStatus {
        match (self, other) {
            (ScrapeStatus::Success, _) | (_, ScrapeStatus::Success) => ScrapeStatus::Success,
            (ScrapeStatus::Failed, _) | (_, ScrapeStatus::Failed) => ScrapeStatus::Failed,
            (ScrapeStatus::Searching, _) | (_, ScrapeStatus::Searching) => ScrapeStatus::Searching,
            (ScrapeStatus::Pending, _) | (_, ScrapeStatus::Pending) => ScrapeStatus::Pending,
            (ScrapeStatus::Skipped, ScrapeStatus::Skipped) => ScrapeStatus::Skipped,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameData {
    pub rom_name: String,
    pub metadata: GameMetadata,
    pub guides: GameGuides,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameMetadata {
    pub status: ScrapeStatus,
    pub name: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<String>,
    pub rating: Option<String>,
    pub image_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameGuides {
    pub status: ScrapeStatus,
    pub count: Option<usize>,
}

pub struct ScrapingProgress {
    pub total: usize,
    pub completed: usize,
    pub current_rom: Option<String>,
    pub success_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ProgressUpdate {
    pub total: usize,
    pub completed: usize,
    pub current_rom: Option<String>,
    pub success_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
    pub message: String,
    pub game_update: Option<GameData>,
}

/// Helper function to send progress update
pub fn send_progress(
    progress_tx: &UnboundedSender<ProgressUpdate>,
    progress: &ScrapingProgress,
    message: String,
    game_update: Option<GameData>,
) {
    // Log to console
    if let Some(ref game_data) = game_update {
        let status = game_data.metadata.status.merge(game_data.guides.status);
        match status {
            ScrapeStatus::Pending => info!("  {} - {}", game_data.rom_name, message),
            ScrapeStatus::Searching => info!("🔍 {} - {}", game_data.rom_name, message),
            ScrapeStatus::Success => info!("✓ {} - {}", game_data.rom_name, message),
            ScrapeStatus::Failed => info!("✗ {} - {}", game_data.rom_name, message),
            ScrapeStatus::Skipped => info!("⏭ {} - {}", game_data.rom_name, message),
        }
    } else {
        info!("{}", message);
    }

    let _ = progress_tx.send(ProgressUpdate {
        total: progress.total,
        completed: progress.completed,
        current_rom: progress.current_rom.clone(),
        success_count: progress.success_count,
        fail_count: progress.fail_count,
        skip_count: progress.skip_count,
        message,
        game_update,
    });
}
