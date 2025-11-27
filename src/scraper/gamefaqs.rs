use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::scraper::{GuidesScraper, ScraperError, ScraperResult};
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFAQsConfig(bool);

/// GameFAQs archive scraper using the gopher protocol
pub struct GameFAQsScraper {
    host: String,
    port: u16,
}

impl GameFAQsScraper {
    pub fn new() -> Self {
        Self {
            host: "gopher.endangeredsoft.org".to_string(),
            port: 70,
        }
    }

    /// Normalize game name to match GameFAQs archive structure
    /// GameFAQs uses lowercase with hyphens
    fn normalized_name(path: &Path) -> String {
        let name = path
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("")
            .to_string();

        // Remove numbers
        static NUMBERS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+[.\)]").unwrap());
        let name = NUMBERS_RE.replace(&name, "").to_string();

        // Remove trailing parenthesis
        static PARENTHESIS_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"[\(\[].+[\)\]]$").unwrap());
        let name = PARENTHESIS_RE.replace(&name, "").to_string();

        let name = name.trim();
        let name = name.to_lowercase();

        // Mario & Luigi -> Mario and Luigi
        let name = name.replace('&', " and ");

        // Pokemon...
        let name = name.replace('é', "e");

        static STRIP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("[^a-z0-9 -]").unwrap());

        STRIP_RE
            .replace_all(&name, "")
            .replace('-', " ")
            .split_whitespace()
            .join("-")
    }

    /// Fetch a text file from the gopher server
    async fn fetch_text(&self, path: &str) -> ScraperResult<String> {
        let stream = TcpStream::connect(format!("{}:{}", self.host, self.port))
            .map_err(|e| ScraperError::Network(format!("Failed to connect: {}", e)))?;

        let mut stream_clone = stream
            .try_clone()
            .map_err(|e| ScraperError::Network(format!("Failed to clone stream: {}", e)))?;

        // Send gopher request
        let request = format!("{}\r\n", path);
        stream_clone
            .write_all(request.as_bytes())
            .map_err(|e| ScraperError::Network(format!("Failed to write request: {}", e)))?;
        stream_clone
            .flush()
            .map_err(|e| ScraperError::Network(format!("Failed to flush: {}", e)))?;

        // Read response
        let reader = BufReader::new(stream);
        let mut content = String::new();
        for line in reader.lines() {
            let line =
                line.map_err(|e| ScraperError::Network(format!("Failed to read line: {}", e)))?;
            content.push_str(&line);
            content.push('\n');
        }

        Ok(content)
    }

    /// Fetch a directory listing from the gopher server
    async fn fetch_directory(&self, path: &str) -> ScraperResult<Vec<GopherEntry>> {
        let stream = TcpStream::connect(format!("{}:{}", self.host, self.port))
            .map_err(|e| ScraperError::Network(format!("Failed to connect: {}", e)))?;

        let mut stream_clone = stream
            .try_clone()
            .map_err(|e| ScraperError::Network(format!("Failed to clone stream: {}", e)))?;

        // Send gopher request
        let request = format!("{}\r\n", path);
        stream_clone
            .write_all(request.as_bytes())
            .map_err(|e| ScraperError::Network(format!("Failed to write request: {}", e)))?;
        stream_clone
            .flush()
            .map_err(|e| ScraperError::Network(format!("Failed to flush: {}", e)))?;

        // Parse directory listing
        let reader = BufReader::new(stream);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line =
                line.map_err(|e| ScraperError::Network(format!("Failed to read line: {}", e)))?;

            // Gopher protocol: lines end with single period
            if line == "." {
                break;
            }

            if let Some(entry) = Self::parse_gopher_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Parse a gopher directory line
    fn parse_gopher_line(line: &str) -> Option<GopherEntry> {
        if line.is_empty() {
            return None;
        }

        let item_type = line.chars().next()?;
        let parts: Vec<&str> = line[1..].split('\t').collect();

        if parts.len() < 2 {
            return None;
        }

        Some(GopherEntry {
            item_type,
            display_name: parts[0].to_string(),
            path: parts[1].to_string(),
        })
    }
}

impl Default for GameFAQsScraper {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GuidesScraper for GameFAQsScraper {
    fn name(&self) -> &'static str {
        "GameFAQs"
    }

    async fn search_game_guides(
        &self,
        path: &Path,
        console: &crate::console::Console,
    ) -> crate::scraper::ScraperResult<Vec<String>> {
        tracing::debug!(
            "Searching GameFAQs guides for '{}' on console '{}' ({:?})",
            path.file_name().and_then(OsStr::to_str).unwrap_or(""),
            console.name,
            console.gamefaqs_archive_id
        );
        // Get the GameFAQs platform identifier from the console
        let platform = console
            .gamefaqs_archive_id
            .as_ref()
            .ok_or(crate::scraper::ScraperError::PlatformNotSupported)?;

        let normalized_name = Self::normalized_name(path);

        // Try to list the game directory
        let path = format!("/gamefaqs-archive/{}/{}", platform, normalized_name);

        match self.fetch_directory(&path).await {
            Ok(entries) => {
                // Filter for text files (type '0' in gopher)
                let guides = entries
                    .into_iter()
                    .filter(|entry| entry.item_type == '0' && entry.path.ends_with(".txt"))
                    .map(|entry| entry.path)
                    .collect();
                Ok(guides)
            }
            Err(_) => {
                // Game not found or no guides available
                Ok(Vec::new())
            }
        }
    }

    async fn download_guide(
        &self,
        guide_path: &str,
        destination: &Path,
    ) -> crate::scraper::ScraperResult<()> {
        let content = self.fetch_text(guide_path).await?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(crate::scraper::ScraperError::Io)?;
        }

        std::fs::write(destination, content).map_err(crate::scraper::ScraperError::Io)?;

        Ok(())
    }
}

#[derive(Debug)]
struct GopherEntry {
    item_type: char, // '0' = text file, '1' = directory
    #[allow(dead_code)]
    display_name: String,
    path: String,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    use test_case::test_case;

    #[test_case("Advance Wars.gba", "advance-wars")]
    #[test_case(
        "Advance Wars 2: Black Hole Rising.gba",
        "advance-wars-2-black-hole-rising"
    )]
    #[test_case("Pokémon: Emerald.gba", "pokemon-emerald")]
    #[test_case("Mario & Luigi - Superstar Saga.gba", "mario-and-luigi-superstar-saga")]
    #[test_case("Mario Kart - Super Circuit.gba", "mario-kart-super-circuit")]
    #[test_case("Mario vs. Donkey Kong.gba", "mario-vs-donkey-kong")]
    #[test_case("Mother 1+2.gba", "mother-12")]
    #[test_case("Mother 3.gba", "mother-3")]
    #[test_case("Ninja Five-O.gba", "ninja-five-o")]
    #[test_case("Spider-Man - Mysterio's Menace.gba", "spider-man-mysterios-menace")]
    #[test_case(
        "Yggdra Union - We'll Never Fight Alone.gba",
        "yggdra-union-well-never-fight-alone"
    )]
    fn test_normalized_name(name: &str, expected: &str) {
        let path = PathBuf::from(name);
        assert_eq!(GameFAQsScraper::normalized_name(&path), expected);
    }

    #[test]
    fn test_parse_gopher_line() {
        let line = "0FAQ_Walkthrough-by--SGibson.txt\t/gamefaqs-archive/gba/advance-wars/FAQ_Walkthrough-by--SGibson.txt\tgopher.endangeredsoft.org\t70";
        let entry = GameFAQsScraper::parse_gopher_line(line).unwrap();

        assert_eq!(entry.item_type, '0');
        assert_eq!(entry.display_name, "FAQ_Walkthrough-by--SGibson.txt");
        assert_eq!(
            entry.path,
            "/gamefaqs-archive/gba/advance-wars/FAQ_Walkthrough-by--SGibson.txt"
        );
    }
}
