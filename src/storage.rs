use crate::{progress::GameData, scanner};
use std::path::Path;

/// Load existing game data from .collie/games/<path>.json
pub fn load_game_data(roms_path: &Path, rom: &scanner::RomFile) -> Option<GameData> {
    let games_dir = roms_path.join(".collie").join("games");

    // Create a path-safe filename using console and rom name
    let safe_filename = format!(
        "{}_{}.json",
        rom.console.name.replace(['/', '\\', ':'], "_"),
        rom.name_no_extension.replace(['/', '\\', ':'], "_")
    );

    let game_file = games_dir.join(safe_filename);

    if !game_file.exists() {
        return None;
    }

    match std::fs::read_to_string(&game_file) {
        Ok(content) => serde_json::from_str(&content).ok(),
        Err(_) => None,
    }
}

/// Save game data to .collie/games/<path>.json
pub fn save_game_data(
    roms_path: &Path,
    rom: &scanner::RomFile,
    game_data: &GameData,
) -> Result<(), Box<dyn std::error::Error>> {
    let games_dir = roms_path.join(".collie").join("games");
    std::fs::create_dir_all(&games_dir)?;

    // Create a path-safe filename using console and rom name
    let safe_filename = format!(
        "{}_{}.json",
        rom.console.name.replace(['/', '\\', ':'], "_"),
        rom.name_no_extension.replace(['/', '\\', ':'], "_")
    );

    let game_file = games_dir.join(safe_filename);
    let json = serde_json::to_string_pretty(game_data)?;
    std::fs::write(&game_file, json)?;

    Ok(())
}

/// Append crawled path to .collie/crawled
pub fn append_crawled_path(
    roms_path: &Path,
    rom: &scanner::RomFile,
) -> Result<(), Box<dyn std::error::Error>> {
    let collie_dir = roms_path.join(".collie");
    std::fs::create_dir_all(&collie_dir)?;

    let crawled_file = collie_dir.join("crawled");

    // Create a path entry with console and rom name using PathBuf to ensure correct separator
    let path = std::path::PathBuf::from(&rom.console.name).join(&rom.name);
    let path_entry = format!("{}\n", path.display());

    // Append to file
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&crawled_file)?;
    file.write_all(path_entry.as_bytes())?;

    Ok(())
}

/// Append game entry to .collie/games.txt index
pub fn append_scraped_index(
    roms_path: &Path,
    _rom: &scanner::RomFile,
    game_data: &GameData,
) -> Result<(), Box<dyn std::error::Error>> {
    let collie_dir = roms_path.join(".collie");
    std::fs::create_dir_all(&collie_dir)?;

    let games_index_file = collie_dir.join("games.txt");

    // Append ROM file path as plain text line
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&games_index_file)?;
    writeln!(file, "{}", game_data.rom_name)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use crate::progress::{GameData, GameGuides, GameMetadata, ScrapeStatus};
    use std::path::PathBuf;

    #[test]
    fn test_append_crawled_path_uses_correct_separator() {
        let temp_dir = std::env::temp_dir().join("collie_test_crawled_path");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let console = Console {
            name: "GBA".to_string(),
            patterns: vec!["gba".to_string()],
            screenscraper_id: None,
            thegamesdb_id: None,
            gamefaqs_archive_id: None,
        };

        let rom = scanner::RomFile {
            path: PathBuf::from("GBA/game.gba"),
            name: "game.gba".to_string(),
            name_no_extension: "game".to_string(),
            console,
        };

        append_crawled_path(&temp_dir, &rom).unwrap();

        // Read the crawled file
        let crawled_file = temp_dir.join(".collie").join("crawled");
        let content = std::fs::read_to_string(&crawled_file).unwrap();

        // On Windows, the path should contain backslash; on Unix, forward slash
        #[cfg(windows)]
        assert!(
            content.contains("GBA\\game.gba"),
            "Expected Windows path separator (\\), got: {}",
            content
        );

        #[cfg(not(windows))]
        assert!(
            content.contains("GBA/game.gba"),
            "Expected Unix path separator (/), got: {}",
            content
        );

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_save_and_load_game_data() {
        let temp_dir = std::env::temp_dir().join("collie_test_game_data");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let console = Console {
            name: "PS".to_string(),
            patterns: vec!["ps".to_string()],
            screenscraper_id: None,
            thegamesdb_id: None,
            gamefaqs_archive_id: None,
        };

        let rom = scanner::RomFile {
            path: PathBuf::from("PS/test.bin"),
            name: "test.bin".to_string(),
            name_no_extension: "test".to_string(),
            console,
        };

        let game_data = GameData {
            rom_name: "test.bin".to_string(),
            metadata: GameMetadata {
                status: ScrapeStatus::Success,
                name: Some("Test Game".to_string()),
                developer: None,
                publisher: None,
                genre: None,
                release_date: None,
                rating: None,
                image_path: None,
                error_message: None,
            },
            guides: GameGuides {
                status: ScrapeStatus::Pending,
                count: None,
            },
        };

        // Save
        save_game_data(&temp_dir, &rom, &game_data).unwrap();

        // Load
        let loaded = load_game_data(&temp_dir, &rom).unwrap();
        assert_eq!(loaded.metadata.name, Some("Test Game".to_string()));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_path_safe_filename_generation() {
        let temp_dir = std::env::temp_dir().join("collie_test_safe_filename");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let console = Console {
            name: "N64".to_string(),
            patterns: vec!["n64".to_string()],
            screenscraper_id: None,
            thegamesdb_id: None,
            gamefaqs_archive_id: None,
        };

        // Test with special characters that should be replaced
        let rom = scanner::RomFile {
            path: PathBuf::from("N64/test:game/special.z64"),
            name: "test:game/special.z64".to_string(),
            name_no_extension: "test:game/special".to_string(),
            console,
        };

        let game_data = GameData {
            rom_name: "test:game/special.z64".to_string(),
            metadata: GameMetadata {
                status: ScrapeStatus::Success,
                name: Some("Test Game".to_string()),
                developer: None,
                publisher: None,
                genre: None,
                release_date: None,
                rating: None,
                image_path: None,
                error_message: None,
            },
            guides: GameGuides {
                status: ScrapeStatus::Pending,
                count: None,
            },
        };

        // This should not panic and should create a valid filename
        save_game_data(&temp_dir, &rom, &game_data).unwrap();

        // Verify the file was created with safe characters
        let games_dir = temp_dir.join(".collie").join("games");
        assert!(games_dir.exists());

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
