pub mod gamefaqs;
pub mod screenscraper;
pub mod thegamesdb;

use std::path::Path;

/// Metadata for a game scraped from a backend
#[derive(Debug, Clone)]
pub struct GameMetadata {
    pub name: String,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<String>,
    pub rating: Option<f32>,
    pub image_url: Option<String>,
    pub thumbnail_url: Option<String>,
}

/// Result of a scraping operation
pub type ScraperResult<T> = Result<T, ScraperError>;

/// Errors that can occur during scraping
#[derive(Debug, thiserror::Error)]
pub enum ScraperError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Game not found")]
    GameNotFound,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Platform not supported")]
    PlatformNotSupported,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Trait for implementing metadata scraping backends
#[async_trait::async_trait]
pub trait MetadataScraper: Send + Sync {
    fn name(&self) -> &'static str;

    /// Authenticate with the scraping service if required
    async fn authenticate(&mut self, username: &str, password: &str) -> ScraperResult<()>;

    /// Search for a game by name and console
    async fn search_game(&self, path: &Path, console: &Console) -> ScraperResult<GameMetadata>;

    /// Get detailed metadata for a specific game
    async fn get_game_metadata(&self, game_id: &str) -> ScraperResult<GameMetadata>;

    /// Download an image to a local path
    async fn download_image(&self, url: &str, destination: &Path) -> ScraperResult<()>;
}

use crate::console::Console;

/// Trait for implementing guide scraping backends
#[async_trait::async_trait]
pub trait GuidesScraper: Send + Sync {
    fn name(&self) -> &'static str;

    /// Search for guides for a specific game
    async fn search_game_guides(
        &self,
        path: &Path,
        console: &Console,
    ) -> ScraperResult<Vec<String>>;

    /// Download a guide to a local path
    async fn download_guide(&self, guide_path: &str, destination: &Path) -> ScraperResult<()>;
}
