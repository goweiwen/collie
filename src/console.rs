use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Console {
    pub name: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub screenscraper_id: Option<i32>,
    #[serde(default)]
    pub thegamesdb_id: Option<i32>,
    #[serde(default)]
    pub gamefaqs_archive_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConsolesConfig {
    pub consoles: Vec<Console>,
}

impl ConsolesConfig {
    /// Load consoles configuration from a TOML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: ConsolesConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load the bundled consoles.toml from the binary
    pub fn from_embedded() -> Result<Self, Box<dyn std::error::Error>> {
        const CONSOLES_TOML: &str = include_str!("../consoles.toml");
        let config: ConsolesConfig = toml::from_str(CONSOLES_TOML)?;
        Ok(config)
    }

    /// Find a console by folder name pattern
    pub fn find_console(&self, folder_name: &str) -> Option<&Console> {
        self.consoles.iter().find(|console| {
            console
                .patterns
                .iter()
                .any(|pattern| folder_name.eq_ignore_ascii_case(pattern))
        })
    }

    /// Get all console patterns
    pub fn all_patterns(&self) -> Vec<String> {
        self.consoles
            .iter()
            .flat_map(|c| c.patterns.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded_consoles() {
        let config = ConsolesConfig::from_embedded().unwrap();
        assert!(!config.consoles.is_empty());
    }

    #[test]
    fn test_find_console() {
        let config = ConsolesConfig::from_embedded().unwrap();

        // Test exact match
        let sfc = config.find_console("SFC").unwrap();
        assert_eq!(sfc.name, "SNES");
        assert_eq!(sfc.screenscraper_id, Some(4));

        // Test case insensitive
        let gb = config.find_console("gb").unwrap();
        assert_eq!(gb.name, "Game Boy");
        assert_eq!(gb.screenscraper_id, Some(9));

        // Test not found
        assert!(config.find_console("NOTEXIST").is_none());
    }
}
