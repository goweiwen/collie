use crate::console::Console;

use super::{GameMetadata, MetadataScraper, ScraperError, ScraperResult};
use async_trait::async_trait;
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, Metadata},
    io::Read,
    path::Path,
};
use tracing::debug;

const SCREENSCRAPER_API_URL: &str = "https://api.screenscraper.fr/api2";
const DEV_ID: &str = "Silenced8261";
const DEV_PASSWORD: &str = "d7jsIKm1Jt7";
const SOFTWARE_NAME: &str = "collie";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenScraperConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default = "default_box_art_type")]
    pub box_art_type: String,
}

fn default_box_art_type() -> String {
    "box-2D".to_string()
}

pub struct ScreenScraper {
    username: Option<String>,
    password: Option<String>,
    box_art_type: String,
    client: reqwest::Client,
}

impl Default for ScreenScraper {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenScraper {
    pub fn new() -> Self {
        Self {
            username: None,
            password: None,
            box_art_type: default_box_art_type(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_box_art_type(mut self, box_art_type: String) -> Self {
        self.box_art_type = box_art_type;
        self
    }

    /// Build API URL with authentication parameters
    fn build_api_url(&self, endpoint: &str, params: &[(&str, &str)]) -> String {
        let mut url = format!("{}/{}", SCREENSCRAPER_API_URL, endpoint);
        url.push_str(&format!(
            "?devid={}&devpassword={}&softname={}",
            DEV_ID, DEV_PASSWORD, SOFTWARE_NAME
        ));

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            url.push_str(&format!("&ssid={}&sspassword={}", username, password));
        }

        url.push_str("&output=json");

        for (key, value) in params {
            url.push_str(&format!("&{}={}", key, value));
        }

        url
    }

    /// Search for a game by name, console, and optional CRC
    async fn search_game_internal(
        &self,
        rom_name: &str,
        console_id: &str,
        rom_size: Option<u64>,
        crc: Option<String>,
    ) -> ScraperResult<serde_json::Value> {
        debug!(
            "Searching ScreenScraper for rom_name: {}, console_id: {}, rom_size: {:?}, crc: {:?}",
            rom_name, console_id, rom_size, crc
        );

        let rom_name_encoded = urlencoding::encode(rom_name);
        let mut params = vec![
            ("romnom", rom_name_encoded.as_ref()),
            ("systemeid", console_id),
            ("romtype", "rom"),
        ];

        let size_string = rom_size.map(|s| s.to_string());
        if let Some(ref size_str) = size_string {
            params.push(("romtaille", size_str.as_str()));
        }

        if let Some(crc) = crc.as_ref() {
            params.push(("crc", crc));
        }

        let url = self.build_api_url("jeuInfos.php", &params);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        let status = response.status();

        let text = response
            .text()
            .await
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        debug!("ScreenScraper response status: {}, body: {}", status, text);

        match status.as_u16() {
            200 => {
                let json: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| ScraperError::ParseError(e.to_string()))?;
                Ok(json)
            }
            403 => Err(ScraperError::AuthenticationFailed),
            404 => Err(ScraperError::GameNotFound),
            429..=431 => Err(ScraperError::RateLimitExceeded),
            _ => Err(ScraperError::Network(text)),
        }
    }

    /// Extract game metadata from API response
    fn parse_game_metadata(&self, json: &serde_json::Value) -> ScraperResult<GameMetadata> {
        let jeu = json
            .get("response")
            .and_then(|r| r.get("jeu"))
            .or_else(|| json.get("jeu"))
            .ok_or_else(|| ScraperError::ParseError("Missing 'jeu' field".to_string()))?;

        let name = jeu
            .get("noms")
            .and_then(|n| n.get(0))
            .and_then(|n| n.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let description = jeu
            .get("synopsis")
            .and_then(|s| s.get(0))
            .and_then(|s| s.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let release_date = jeu
            .get("dates")
            .and_then(|d| d.get(0))
            .and_then(|d| d.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let developer = jeu
            .get("developpeur")
            .and_then(|d| d.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let publisher = jeu
            .get("editeur")
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let genre = jeu
            .get("genres")
            .and_then(|g| g.get(0))
            .and_then(|g| g.get("noms"))
            .and_then(|n| n.get(0))
            .and_then(|n| n.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let players = jeu
            .get("joueurs")
            .and_then(|j| j.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let rating = jeu
            .get("classifications")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .and_then(|s| s.parse::<f32>().ok());

        Ok(GameMetadata {
            name,
            description,
            release_date,
            developer,
            publisher,
            genre,
            players,
            rating,
            image_url: None,
            thumbnail_url: None,
        })
    }

    /// Get media URL for a specific type and region preferences
    fn get_media_url(
        &self,
        json: &serde_json::Value,
        media_type: &str,
        region_preferences: &[&str],
    ) -> Option<String> {
        let medias = json
            .get("response")
            .and_then(|r| r.get("jeu"))
            .and_then(|j| j.get("medias"))
            .or_else(|| json.get("jeu").and_then(|j| j.get("medias")))?;

        let medias_array = medias.as_array()?;

        // Filter by media type
        let mut matching_medias: Vec<_> = medias_array
            .iter()
            .filter(|m| {
                m.get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t == media_type)
                    .unwrap_or(false)
            })
            .collect();

        // Sort by region preference
        matching_medias.sort_by_key(|m| {
            let region = m
                .get("region")
                .and_then(|r| r.as_str())
                .unwrap_or("unknown");

            region_preferences
                .iter()
                .position(|&r| r == region)
                .unwrap_or(999)
        });

        matching_medias
            .first()
            .and_then(|m| m.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
    }
}

#[async_trait]
impl MetadataScraper for ScreenScraper {
    fn name(&self) -> &'static str {
        "ScreenScraper"
    }

    async fn authenticate(&mut self, username: &str, password: &str) -> ScraperResult<()> {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        Ok(())
    }

    async fn search_game(&self, path: &Path, console: &Console) -> ScraperResult<GameMetadata> {
        let Some(console_id) = console.screenscraper_id.as_ref() else {
            return Err(ScraperError::PlatformNotSupported);
        };

        let Some(file_name) = path.file_name() else {
            return Err(ScraperError::GameNotFound);
        };
        let file_name = file_name.to_string_lossy();

        let mut size = None;
        let mut crc = None;

        if let Ok(mut file) = File::open(path) {
            size = file.metadata().as_ref().map(Metadata::len).ok();

            let mut hasher = Hasher::new();
            let mut buffer = [0u8; 8192];
            while let Ok(bytes_read) = file.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            crc = Some(format!("{:x}", hasher.finalize()));
        }

        let result = self
            .search_game_internal(&file_name, &console_id.to_string(), size, crc)
            .await?;

        let mut metadata = self.parse_game_metadata(&result)?;

        // Extract image URL with region preferences
        let region_preferences = vec!["us", "wor", "eu", "jp", "ss"];
        let image_url = self.get_media_url(&result, &self.box_art_type, &region_preferences);
        metadata.image_url = image_url;

        Ok(metadata)
    }

    async fn get_game_metadata(&self, _game_id: &str) -> ScraperResult<GameMetadata> {
        // For ScreenScraper, we get metadata from search
        // This could be extended if we need to fetch by game ID
        Err(ScraperError::ParseError(
            "Not implemented for ScreenScraper".to_string(),
        ))
    }

    async fn download_image(&self, url: &str, destination: &Path) -> ScraperResult<()> {
        // Add max dimensions to URL
        let url_with_params = if url.contains('?') {
            format!("{}&maxwidth=250&maxheight=360", url)
        } else {
            format!("{}?maxwidth=250&maxheight=360", url)
        };

        let response = self
            .client
            .get(&url_with_params)
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
