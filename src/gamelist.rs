use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "gameList")]
pub struct GameList {
    #[serde(default, rename = "game")]
    pub games: Vec<Game>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub path: PathBuf,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releasedate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "guide")]
    pub guides: Option<Vec<PathBuf>>,
}

impl GameList {
    /// Load gamelist.xml from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let gamelist = quick_xml::de::from_str(&content)?;
        Ok(gamelist)
    }

    /// Save gamelist.xml to a file
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let xml = quick_xml::se::to_string_with_root("gameList", self)?;

        // Pretty print the XML
        let formatted = Self::format_xml(&xml);

        fs::write(path, formatted)?;
        Ok(())
    }

    /// Simple XML formatter
    fn format_xml(xml: &str) -> String {
        let mut formatted = String::new();
        let mut indent: usize = 0;
        let mut i = 0;
        let chars: Vec<char> = xml.chars().collect();

        while i < chars.len() {
            if chars[i] == '<' {
                // Look ahead to determine tag type
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '>' {
                    j += 1;
                }

                if j < chars.len() {
                    let tag_content: String = chars[i + 1..j].iter().collect();

                    // XML declaration
                    if tag_content.starts_with("?xml") {
                        formatted.push_str(&chars[i..=j].iter().collect::<String>());
                        formatted.push('\n');
                        i = j + 1;
                        continue;
                    }

                    // Closing tag
                    if tag_content.starts_with('/') {
                        indent = indent.saturating_sub(1);
                        formatted.push_str(&"  ".repeat(indent));
                        formatted.push_str(&chars[i..=j].iter().collect::<String>());
                        formatted.push('\n');
                        i = j + 1;
                        continue;
                    }

                    // Self-closing tag
                    if tag_content.ends_with('/') {
                        formatted.push_str(&"  ".repeat(indent));
                        formatted.push_str(&chars[i..=j].iter().collect::<String>());
                        formatted.push('\n');
                        i = j + 1;
                        continue;
                    }

                    // Opening tag - check if it has text content before closing tag
                    let mut has_text = false;
                    let mut k = j + 1;
                    while k < chars.len() && chars[k] != '<' {
                        if !chars[k].is_whitespace() {
                            has_text = true;
                            break;
                        }
                        k += 1;
                    }

                    formatted.push_str(&"  ".repeat(indent));
                    formatted.push_str(&chars[i..=j].iter().collect::<String>());

                    if has_text {
                        // Inline content - don't add newline, collect text until closing tag
                        i = j + 1;
                        while i < chars.len() && chars[i] != '<' {
                            formatted.push(chars[i]);
                            i += 1;
                        }
                    } else {
                        // No inline content - newline and indent
                        formatted.push('\n');
                        indent += 1;
                        i = j + 1;
                    }
                    continue;
                }
            }

            i += 1;
        }

        formatted
    }

    /// Add or update a game entry
    pub fn add_or_update_game(&mut self, game: Game) {
        // Remove existing entry with same path
        self.games.retain(|g| g.path != game.path);

        // Add new entry
        self.games.push(game);

        // Sort by path for consistency
        self.games.sort_by(|a, b| a.path.cmp(&b.path));
    }

    /// Find a game by path
    pub fn find_game(&self, path: &Path) -> Option<&Game> {
        self.games.iter().find(|g| g.path == path)
    }
}

impl Game {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            desc: None,
            image: None,
            rating: None,
            releasedate: None,
            developer: None,
            publisher: None,
            genre: None,
            players: None,
            guides: None,
        }
    }

    /// Set the description
    pub fn with_desc(mut self, desc: Option<String>) -> Self {
        self.desc = desc;
        self
    }

    /// Set the image path (relative to gamelist.xml)
    pub fn with_image(mut self, image: Option<PathBuf>) -> Self {
        self.image = image;
        self
    }

    /// Set the rating (0.0 to 1.0)
    pub fn with_rating(mut self, rating: Option<f32>) -> Self {
        self.rating = rating.map(|r| format!("{:.1}", r));
        self
    }

    /// Set the release date (YYYYMMDDTHHMMSS format)
    pub fn with_releasedate(mut self, date: Option<String>) -> Self {
        self.releasedate = date.map(|d| Self::format_date(&d));
        self
    }

    /// Set the developer
    pub fn with_developer(mut self, developer: Option<String>) -> Self {
        self.developer = developer;
        self
    }

    /// Set the publisher
    pub fn with_publisher(mut self, publisher: Option<String>) -> Self {
        self.publisher = publisher;
        self
    }

    /// Set the genre
    pub fn with_genre(mut self, genre: Option<String>) -> Self {
        self.genre = genre;
        self
    }

    /// Set the number of players
    pub fn with_players(mut self, players: Option<String>) -> Self {
        self.players = players;
        self
    }

    /// Set the guides
    pub fn with_guides(mut self, guides: Option<Vec<PathBuf>>) -> Self {
        self.guides = guides;
        self
    }

    /// Format date to YYYYMMDDTHHMMSS
    fn format_date(date: &str) -> String {
        // Try to parse common date formats and convert to YYYYMMDDTHHMMSS
        // Input could be: "1990-11-21", "1990", "21/11/1990", etc.

        // If already in the right format, return it
        if date.contains('T') && date.len() >= 15 {
            return date.to_string();
        }

        // Try to extract year, month, day
        if let Some((year, month, day)) = Self::parse_date(date) {
            format!("{}{}{}T000000", year, month, day)
        } else {
            // Default to just the input if we can't parse it
            format!("{}0101T000000", date)
        }
    }

    /// Parse various date formats
    fn parse_date(date: &str) -> Option<(String, String, String)> {
        // Remove any whitespace
        let date = date.trim();

        // Try YYYY-MM-DD
        if let Some(parts) = date.split('-').collect::<Vec<_>>().get(0..3)
            && parts.len() == 3
        {
            return Some((
                parts[0].to_string(),
                format!("{:02}", parts[1].parse::<u32>().ok()?),
                format!("{:02}", parts[2].parse::<u32>().ok()?),
            ));
        }

        // Try YYYY/MM/DD
        if let Some(parts) = date.split('/').collect::<Vec<_>>().get(0..3)
            && parts.len() == 3
        {
            // Could be DD/MM/YYYY or YYYY/MM/DD
            if parts[0].len() == 4 {
                // YYYY/MM/DD
                return Some((
                    parts[0].to_string(),
                    format!("{:02}", parts[1].parse::<u32>().ok()?),
                    format!("{:02}", parts[2].parse::<u32>().ok()?),
                ));
            } else {
                // DD/MM/YYYY
                return Some((
                    parts[2].to_string(),
                    format!("{:02}", parts[1].parse::<u32>().ok()?),
                    format!("{:02}", parts[0].parse::<u32>().ok()?),
                ));
            }
        }

        // Try just YYYY
        if date.len() == 4 && date.parse::<u32>().is_ok() {
            return Some((date.to_string(), "01".to_string(), "01".to_string()));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_game() {
        let game = Game::new(PathBuf::from("./Game.gba"), "Test Game".to_string())
            .with_desc(Some("A test game".to_string()))
            .with_rating(Some(0.85))
            .with_releasedate(Some("2000-01-01".to_string()))
            .with_developer(Some("Test Dev".to_string()))
            .with_publisher(Some("Test Pub".to_string()))
            .with_genre(Some("Action".to_string()))
            .with_players(Some("1-2".to_string()));

        assert_eq!(game.name, "Test Game");
        assert_eq!(game.desc, Some("A test game".to_string()));
        assert_eq!(game.rating, Some("0.9".to_string())); // 0.85 rounds to 0.9 with .1 precision
        assert_eq!(game.releasedate, Some("20000101T000000".to_string()));
    }

    #[test]
    fn test_date_formatting() {
        assert_eq!(Game::format_date("2000-01-15"), "20000115T000000");
        assert_eq!(Game::format_date("2000/01/15"), "20000115T000000");
        assert_eq!(Game::format_date("15/01/2000"), "20000115T000000");
        assert_eq!(Game::format_date("2000"), "20000101T000000");
    }

    #[test]
    fn test_gamelist_add_update() {
        let mut gamelist = GameList::default();

        let game1 = Game::new(PathBuf::from("./Game1.gba"), "Game 1".to_string());
        let game2 = Game::new(PathBuf::from("./Game2.gba"), "Game 2".to_string());

        gamelist.add_or_update_game(game1.clone());
        gamelist.add_or_update_game(game2.clone());

        assert_eq!(gamelist.games.len(), 2);

        // Update game1
        let game1_updated = Game::new(PathBuf::from("./Game1.gba"), "Game 1 Updated".to_string());
        gamelist.add_or_update_game(game1_updated);

        assert_eq!(gamelist.games.len(), 2);
        assert_eq!(gamelist.games[0].name, "Game 1 Updated");
    }

    #[test]
    fn test_serialize_gamelist() {
        let mut gamelist = GameList::default();

        let game = Game::new(PathBuf::from("./TestGame.gba"), "Test Game".to_string())
            .with_desc(Some("A description".to_string()))
            .with_rating(Some(0.9))
            .with_developer(Some("Dev".to_string()));

        gamelist.add_or_update_game(game);

        let xml = quick_xml::se::to_string_with_root("gameList", &gamelist).unwrap();
        assert!(xml.contains("<name>Test Game</name>"));
        assert!(xml.contains("<desc>A description</desc>"));
        assert!(xml.contains("<rating>0.9</rating>"));
    }
}
