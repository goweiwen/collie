use std::path::Path;
use crate::{scanner, progress::GameData};

/// Load existing game data from .collie/games/<path>.json
pub fn load_game_data(
    roms_path: &Path,
    rom: &scanner::RomFile,
) -> Option<GameData> {
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

    // Create a path entry with console and rom name
    let path_entry = format!("{}/{}\n", rom.console.name, rom.name);

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
