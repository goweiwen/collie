use crate::console::Console;

use super::{GameMetadata, MetadataScraper, ScraperError, ScraperResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

const THEGAMESDB_API_URL: &str = "https://api.thegamesdb.net/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheGamesDBConfig {
    #[serde(rename = "apiKey")]
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
struct GamesDBResponse {
    data: GamesDBData,
    #[serde(default)]
    include: Option<GamesDBIncludes>,
}

#[derive(Debug, Deserialize)]
struct GamesDBData {
    #[serde(default)]
    games: Vec<GamesDBGame>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GamesDBGame {
    id: u64,
    game_title: String,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    platform: Option<u64>,
    #[serde(default)]
    players: Option<u32>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    rating: Option<String>,
    #[serde(default)]
    developers: Option<Vec<u64>>,
    #[serde(default)]
    publishers: Option<Vec<u64>>,
    #[serde(default)]
    genres: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GamesDBIncludes {
    #[serde(default)]
    boxart: Option<GamesDBBoxart>,
    #[serde(default)]
    platform: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GamesDBBoxart {
    base_url: String,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GamesDBImageData {
    #[serde(default)]
    boxart: Option<Vec<GamesDBImage>>,
    #[serde(default)]
    screenshot: Option<Vec<GamesDBImage>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GamesDBImage {
    id: u64,
    #[serde(rename = "type")]
    image_type: String,
    filename: String,
}

pub struct TheGamesDB {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl Default for TheGamesDB {
    fn default() -> Self {
        Self::new()
    }
}

impl TheGamesDB {
    pub fn new() -> Self {
        Self {
            api_key: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            api_key: Some(api_key),
            client: reqwest::Client::new(),
        }
    }

    fn get_api_key(&self) -> ScraperResult<&str> {
        self.api_key
            .as_deref()
            .ok_or(ScraperError::AuthenticationFailed)
    }

    /// Platform ID mapping for common systems
    /// See: https://api.thegamesdb.net/v1/Platforms
    pub fn map_screenscraper_to_thegamesdb(screenscraper_id: &str) -> Option<&'static str> {
        match screenscraper_id {
            "3" => Some("7"),   // NES
            "4" => Some("6"),   // SNES
            "1" => Some("36"),  // Sega Genesis
            "2" => Some("35"),  // Sega Master System
            "9" => Some("4"),   // Game Boy
            "10" => Some("5"),  // Game Boy Color
            "12" => Some("12"), // Game Boy Advance
            "15" => Some("8"),  // Nintendo DS
            "57" => Some("10"), // PlayStation
            "58" => Some("11"), // PlayStation 2
            _ => None,
        }
    }
}

#[async_trait]
impl MetadataScraper for TheGamesDB {
    fn name(&self) -> &'static str {
        "TheGamesDB"
    }

    async fn authenticate(&mut self, username: &str, _password: &str) -> ScraperResult<()> {
        // TheGamesDB uses API key, not username/password
        // We'll treat the username as the API key
        self.api_key = Some(username.to_string());
        Ok(())
    }

    async fn search_game(&self, path: &Path, console: &Console) -> ScraperResult<GameMetadata> {
        let Some(console_id) = console.thegamesdb_id.as_ref() else {
            return Err(ScraperError::PlatformNotSupported);
        };

        let api_key = self.get_api_key()?;

        // Map ScreenScraper console ID to TheGamesDB platform ID
        let platform_id = Self::map_screenscraper_to_thegamesdb(&console_id.to_string());

        let game_name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or(ScraperError::GameNotFound)?;

        let mut url = format!(
            "{}/Games/ByGameName?apikey={}&name={}",
            THEGAMESDB_API_URL,
            api_key,
            urlencoding::encode(game_name)
        );

        if let Some(platform) = platform_id {
            url.push_str(&format!("&filter[platform]={}", platform));
        }

        url.push_str("&include=boxart");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        if response.status() == 403 {
            return Err(ScraperError::RateLimitExceeded);
        }

        if !response.status().is_success() {
            return Err(ScraperError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let data: GamesDBResponse = response
            .json()
            .await
            .map_err(|e| ScraperError::ParseError(e.to_string()))?;

        if data.data.games.is_empty() {
            return Err(ScraperError::GameNotFound);
        }

        let base_url = data
            .include
            .as_ref()
            .and_then(|i| i.boxart.as_ref())
            .map(|b| b.base_url.clone())
            .unwrap_or_default();

        let Some(game) = data.data.games.into_iter().next() else {
            return Err(ScraperError::GameNotFound);
        };

        let mut metadata = GameMetadata {
            name: game.game_title,
            description: game.overview,
            release_date: game.release_date,
            developer: None,
            publisher: None,
            genre: None,
            players: game.players.map(|p| p.to_string()),
            rating: game.rating.and_then(|r| r.parse::<f32>().ok()),
            image_url: None,
            thumbnail_url: None,
        };

        // Try to get boxart URL
        if let Some(ref includes) = data.include
            && let Some(ref boxart) = includes.boxart
        {
            // Try to find boxart for this game in the data
            if let Some(game_images) = boxart.data.get(game.id.to_string())
                && let Some(images) = game_images.as_array()
            {
                for image in images {
                    if let Some(filename) = image.get("filename").and_then(|f| f.as_str())
                        && image.get("side").and_then(|s| s.as_str()) == Some("front")
                    {
                        metadata.image_url = Some(format!("{}{}", base_url, filename));
                        break;
                    }
                }
                // If no front cover, use any boxart
                if metadata.image_url.is_none()
                    && let Some(image) = images.first()
                    && let Some(filename) = image.get("filename").and_then(|f| f.as_str())
                {
                    metadata.image_url = Some(format!("{}{}", base_url, filename));
                }
            }
        }

        Ok(metadata)
    }

    async fn get_game_metadata(&self, game_id: &str) -> ScraperResult<GameMetadata> {
        let api_key = self.get_api_key()?;

        let url = format!(
            "{}/Games/ByGameID?apikey={}&id={}&include=boxart",
            THEGAMESDB_API_URL, api_key, game_id
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        if response.status() == 403 {
            return Err(ScraperError::RateLimitExceeded);
        }

        let data: GamesDBResponse = response
            .json()
            .await
            .map_err(|e| ScraperError::ParseError(e.to_string()))?;

        let game = data.data.games.first().ok_or(ScraperError::GameNotFound)?;

        Ok(GameMetadata {
            name: game.game_title.clone(),
            description: game.overview.clone(),
            release_date: game.release_date.clone(),
            developer: None,
            publisher: None,
            genre: None,
            players: game.players.map(|p| p.to_string()),
            rating: game.rating.clone().and_then(|r| r.parse::<f32>().ok()),
            image_url: None,
            thumbnail_url: None,
        })
    }

    async fn download_image(&self, url: &str, destination: &Path) -> ScraperResult<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        tokio::fs::write(destination, bytes).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
