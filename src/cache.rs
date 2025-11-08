use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Maximum number of recent game results to keep in cache
const MAX_CACHED_RESULTS: usize = 10;

/// Tracks the overall progress of a scraping session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeProgress {
    pub total: usize,
    pub completed: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
    pub current_rom: Option<String>,
}

/// A simplified game result for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub rom_name: String,
    pub console: String,
    pub metadata_status: String,
    pub guides_status: String,
    pub timestamp: u64,
}

/// Filesystem-based cache for tracking games that failed to scrape or have missing data
/// Uses a directory structure with marker files instead of keeping everything in memory
#[derive(Debug)]
pub struct ScrapeCache {
    pub cache_dir: PathBuf,
}

impl ScrapeCache {
    /// Create a new cache using the default cache directory
    pub fn new(roms_dir: &Path) -> Self {
        Self {
            cache_dir: Self::default_cache_dir(roms_dir),
        }
    }

    /// Create a cache using a custom directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Initialize the cache directory structure
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(self.metadata_dir())?;
        std::fs::create_dir_all(self.guides_dir())?;
        Ok(())
    }

    fn metadata_dir(&self) -> PathBuf {
        self.cache_dir.join("metadata_not_found")
    }

    fn guides_dir(&self) -> PathBuf {
        self.cache_dir.join("guides_not_found")
    }

    fn progress_file(&self) -> PathBuf {
        self.cache_dir.join("progress.json")
    }

    fn results_file(&self) -> PathBuf {
        self.cache_dir.join("results.json")
    }

    /// Get a safe filesystem path for a console/rom combination
    fn get_cache_file_path(&self, base_dir: &Path, console: &str, rom_name: &str) -> PathBuf {
        // Sanitize console and rom_name for filesystem use
        let console_safe = console.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let rom_safe = rom_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");

        base_dir
            .join(&console_safe)
            .join(format!("{}.marker", rom_safe))
    }

    /// Check if metadata scraping failed for this ROM
    pub fn has_metadata_failed(&self, console: &str, rom_name: &str) -> bool {
        let path = self.get_cache_file_path(&self.metadata_dir(), console, rom_name);
        path.exists()
    }

    /// Check if guides were not found for this ROM
    pub fn has_guides_failed(&self, console: &str, rom_name: &str) -> bool {
        let path = self.get_cache_file_path(&self.guides_dir(), console, rom_name);
        path.exists()
    }

    /// Mark metadata as not found for this ROM
    pub fn mark_metadata_not_found(&mut self, console: &str, rom_name: &str) {
        let path = self.get_cache_file_path(&self.metadata_dir(), console, rom_name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, "");
    }

    /// Mark guides as not found for this ROM
    pub fn mark_guides_not_found(&mut self, console: &str, rom_name: &str) {
        let path = self.get_cache_file_path(&self.guides_dir(), console, rom_name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, "");
    }

    /// Remove a ROM from the metadata cache (e.g., if successfully scraped)
    pub fn clear_metadata_failed(&mut self, console: &str, rom_name: &str) {
        let path = self.get_cache_file_path(&self.metadata_dir(), console, rom_name);
        let _ = std::fs::remove_file(path);
    }

    /// Remove a ROM from the guides cache (e.g., if successfully scraped)
    pub fn clear_guides_failed(&mut self, console: &str, rom_name: &str) {
        let path = self.get_cache_file_path(&self.guides_dir(), console, rom_name);
        let _ = std::fs::remove_file(path);
    }

    /// Save scraping progress to cache
    pub fn save_progress(
        &self,
        progress: &ScrapeProgress,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(progress)?;
        std::fs::write(self.progress_file(), json)?;
        Ok(())
    }

    /// Load scraping progress from cache
    pub fn load_progress(&self) -> Option<ScrapeProgress> {
        let path = self.progress_file();
        if !path.exists() {
            return None;
        }

        let contents = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Add a game result to the cache, keeping only the last 10 results
    pub fn add_result(&mut self, result: GameResult) -> Result<(), Box<dyn std::error::Error>> {
        let mut results = self.load_results();

        // Add the new result at the beginning
        results.insert(0, result);

        // Keep only the last MAX_CACHED_RESULTS results
        results.truncate(MAX_CACHED_RESULTS);

        // Save back to file
        let json = serde_json::to_string_pretty(&results)?;
        std::fs::write(self.results_file(), json)?;
        Ok(())
    }

    /// Load the last 10 game results from cache
    pub fn load_results(&self) -> Vec<GameResult> {
        let path = self.results_file();
        if !path.exists() {
            return Vec::new();
        }

        let contents = std::fs::read_to_string(path).ok();
        contents
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    /// Clear progress and results cache (useful when starting a new scraping session)
    pub fn clear_session_cache(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = std::fs::remove_file(self.progress_file());
        let _ = std::fs::remove_file(self.results_file());
        Ok(())
    }

    /// Delete the entire cache directory and all its contents
    pub fn clear_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get default cache directory path
    pub fn default_cache_dir(roms_dir: &Path) -> PathBuf {
        roms_dir.join(".collie").join("cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        // Use a temp directory for testing
        let temp_dir = std::env::temp_dir().join("collie_test_cache");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean up from previous tests

        let mut cache = ScrapeCache::with_cache_dir(temp_dir.clone());
        cache.init().unwrap();

        // Initially nothing is cached
        assert!(!cache.has_metadata_failed("NES", "Super Mario Bros"));
        assert!(!cache.has_guides_failed("NES", "Super Mario Bros"));

        // Mark as failed
        cache.mark_metadata_not_found("NES", "Super Mario Bros");
        cache.mark_guides_not_found("NES", "Super Mario Bros");

        assert!(cache.has_metadata_failed("NES", "Super Mario Bros"));
        assert!(cache.has_guides_failed("NES", "Super Mario Bros"));

        // Different console/rom should not be affected
        assert!(!cache.has_metadata_failed("SNES", "Super Mario Bros"));
        assert!(!cache.has_metadata_failed("NES", "Zelda"));

        // Clear cache
        cache.clear_metadata_failed("NES", "Super Mario Bros");
        assert!(!cache.has_metadata_failed("NES", "Super Mario Bros"));
        assert!(cache.has_guides_failed("NES", "Super Mario Bros"));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sanitization() {
        let temp_dir = std::env::temp_dir().join("collie_test_cache_sanitize");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut cache = ScrapeCache::with_cache_dir(temp_dir.clone());
        cache.init().unwrap();

        // Test with special characters
        cache.mark_metadata_not_found("Console/With:Slashes", "ROM*With?Special<Chars>");
        assert!(cache.has_metadata_failed("Console/With:Slashes", "ROM*With?Special<Chars>"));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_progress_cache() {
        let temp_dir = std::env::temp_dir().join("collie_test_progress");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let cache = ScrapeCache::with_cache_dir(temp_dir.clone());
        cache.init().unwrap();

        // Initially no progress
        assert!(cache.load_progress().is_none());

        // Save progress
        let progress = ScrapeProgress {
            total: 100,
            completed: 50,
            success_count: 40,
            fail_count: 5,
            skip_count: 5,
            current_rom: Some("test_rom.zip".to_string()),
        };
        cache.save_progress(&progress).unwrap();

        // Load progress
        let loaded = cache.load_progress().unwrap();
        assert_eq!(loaded.total, 100);
        assert_eq!(loaded.completed, 50);
        assert_eq!(loaded.success_count, 40);
        assert_eq!(loaded.current_rom, Some("test_rom.zip".to_string()));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_results_cache() {
        let temp_dir = std::env::temp_dir().join("collie_test_results");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut cache = ScrapeCache::with_cache_dir(temp_dir.clone());
        cache.init().unwrap();

        // Initially no results
        assert!(cache.load_results().is_empty());

        // Add results
        for i in 0..15 {
            let result = GameResult {
                rom_name: format!("game_{}.zip", i),
                console: "NES".to_string(),
                metadata_status: "success".to_string(),
                guides_status: "failed".to_string(),
                timestamp: i,
            };
            cache.add_result(result).unwrap();
        }

        // Should only keep last 10
        let results = cache.load_results();
        assert_eq!(results.len(), 10);

        // Most recent should be first
        assert_eq!(results[0].rom_name, "game_14.zip");
        assert_eq!(results[9].rom_name, "game_5.zip");

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clear_session_cache() {
        let temp_dir = std::env::temp_dir().join("collie_test_clear_session");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut cache = ScrapeCache::with_cache_dir(temp_dir.clone());
        cache.init().unwrap();

        // Add some data
        let progress = ScrapeProgress {
            total: 100,
            completed: 50,
            success_count: 40,
            fail_count: 5,
            skip_count: 5,
            current_rom: Some("test.zip".to_string()),
        };
        cache.save_progress(&progress).unwrap();

        let result = GameResult {
            rom_name: "test.zip".to_string(),
            console: "NES".to_string(),
            metadata_status: "success".to_string(),
            guides_status: "success".to_string(),
            timestamp: 123456,
        };
        cache.add_result(result).unwrap();

        // Clear session cache
        cache.clear_session_cache().unwrap();

        // Verify cleared
        assert!(cache.load_progress().is_none());
        assert!(cache.load_results().is_empty());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clear_all() {
        let temp_dir = std::env::temp_dir().join("collie_test_clear_all");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let mut cache = ScrapeCache::with_cache_dir(temp_dir.clone());
        cache.init().unwrap();

        // Add various types of cache data
        cache.mark_metadata_not_found("NES", "Super Mario Bros");
        cache.mark_guides_not_found("SNES", "Zelda");

        let progress = ScrapeProgress {
            total: 100,
            completed: 50,
            success_count: 40,
            fail_count: 5,
            skip_count: 5,
            current_rom: Some("test.zip".to_string()),
        };
        cache.save_progress(&progress).unwrap();

        let result = GameResult {
            rom_name: "test.zip".to_string(),
            console: "NES".to_string(),
            metadata_status: "success".to_string(),
            guides_status: "success".to_string(),
            timestamp: 123456,
        };
        cache.add_result(result).unwrap();

        // Verify cache exists
        assert!(cache.cache_dir.exists());
        assert!(cache.has_metadata_failed("NES", "Super Mario Bros"));

        // Clear all cache
        cache.clear_all().unwrap();

        // Verify cache directory is gone
        assert!(!cache.cache_dir.exists());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
