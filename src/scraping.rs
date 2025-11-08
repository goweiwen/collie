use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, warn};

use crate::backoff::BackoffState;
use crate::image::resize_image;
use crate::progress::{GameData, ProgressUpdate, ScrapeStatus, ScrapingProgress, send_progress};
use crate::scanner;
use crate::scraper::{GuidesScraper, MetadataScraper};
use crate::storage::load_game_data;

pub struct ScrapingConfig {
    pub roms_path: PathBuf,
    pub images_folder: String,
    pub guides_folder: String,
    pub box_art_width: Option<u32>,
    pub skip_cache: bool,
}

/// Scrape game metadata from multiple scrapers with fallback
pub async fn scrape_game_metadata(
    scrapers: &[Box<dyn MetadataScraper>],
    rom: &scanner::RomFile,
    config: &ScrapingConfig,
    game_data: &mut GameData,
    progress: &ScrapingProgress,
    progress_tx: &UnboundedSender<ProgressUpdate>,
    backoff: &mut BackoffState,
) -> bool {
    let console_dir = rom.path.parent().unwrap();
    let image_path = console_dir
        .join(&config.images_folder)
        .join(format!("{}.png", rom.name_no_extension));

    // Check if image already exists (skip this check if skip_cache is enabled)
    if !config.skip_cache && !scrapers.is_empty() && image_path.exists() {
        // Load existing game data to populate metadata
        if let Some(existing) = load_game_data(&config.roms_path, rom) {
            game_data.metadata.name = existing.metadata.name;
            game_data.metadata.developer = existing.metadata.developer;
            game_data.metadata.publisher = existing.metadata.publisher;
            game_data.metadata.genre = existing.metadata.genre;
            game_data.metadata.release_date = existing.metadata.release_date;
            game_data.metadata.rating = existing.metadata.rating;
        }

        game_data.metadata.status = ScrapeStatus::Skipped;

        if let Ok(rel_to_roms) = image_path.strip_prefix(&config.roms_path) {
            let api_path = format!("/api/images/{}", rel_to_roms.display());
            game_data.metadata.image_path = Some(api_path);
        }

        send_progress(
            progress_tx,
            progress,
            "Image already exists, skipping metadata scraping".to_string(),
            Some(game_data.clone()),
        );

        return true; // Skip metadata scraping
    }

    if scrapers.is_empty() {
        return false; // No scrapers configured
    }

    // Search for the game using scrapers with fallback
    game_data.metadata.status = ScrapeStatus::Searching;

    send_progress(
        progress_tx,
        progress,
        format!("Searching for: {}", rom.name),
        Some(game_data.clone()),
    );

    let mut all_not_found = true; // Track if all scrapers returned GameNotFound
    let mut tried_any = false;

    for scraper in scrapers {
        send_progress(
            progress_tx,
            progress,
            format!("Trying {} for: {}", scraper.name(), rom.name),
            None,
        );

        tried_any = true;

        let metadata = match scraper.search_game(&rom.path, &rom.console).await {
            Ok(metadata) => {
                // Success - reset backoff for this scraper
                backoff.reset(scraper.name());
                metadata
            }
            Err(e) => {
                use crate::scraper::ScraperError;

                // Check if this is a GameNotFound error
                if matches!(e, ScraperError::GameNotFound) {
                    warn!(
                        "Scraper {} could not find game: {}",
                        scraper.name(),
                        rom.name
                    );
                    // Keep all_not_found as true
                } else if matches!(e, ScraperError::RateLimitExceeded) {
                    // Apply exponential backoff immediately
                    warn!(
                        "Scraper {} rate limited for game: {}. Applying backoff...",
                        scraper.name(),
                        rom.name
                    );
                    backoff
                        .apply_backoff(scraper.name(), progress_tx, progress)
                        .await;
                    // This was not a "not found" error, so don't cache
                    all_not_found = false;
                } else {
                    warn!(
                        "Scraper {} failed for {}: {:?}",
                        scraper.name(),
                        rom.name,
                        e
                    );
                    // This was not a "not found" error, so don't cache
                    all_not_found = false;
                }
                continue;
            }
        };

        game_data.metadata.name = Some(metadata.name.clone());
        game_data.metadata.developer = metadata.developer.clone();
        game_data.metadata.publisher = metadata.publisher.clone();
        game_data.metadata.genre = metadata.genre.clone();
        game_data.metadata.release_date = metadata.release_date.clone();
        game_data.metadata.rating = metadata.rating.map(|r| format!("{:.1}", r));

        send_progress(
            progress_tx,
            progress,
            format!("{}: Found {}", scraper.name(), metadata.name),
            Some(game_data.clone()),
        );

        if let Some(ref image_url) = metadata.image_url {
            if let Some(parent) = image_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                error!("Failed to create directory: {}", e);
                continue;
            }

            match scraper.download_image(image_url, &image_path).await {
                Ok(_) => {
                    send_progress(progress_tx, progress, "Downloaded image".to_string(), None);
                    if let Some(width) = config.box_art_width
                        && let Err(e) = resize_image(&image_path, width)
                    {
                        error!("Failed to resize image: {}", e);
                    }
                    game_data.metadata.status = ScrapeStatus::Success;

                    // Set the image path for the frontend
                    if let Ok(rel_to_roms) = image_path.strip_prefix(&config.roms_path) {
                        let api_path = format!("/api/images/{}", rel_to_roms.display());
                        game_data.metadata.image_path = Some(api_path);
                    }
                }
                Err(e) => {
                    send_progress(
                        progress_tx,
                        progress,
                        format!("Failed to download image: {}", e),
                        None,
                    );
                    game_data.metadata.status = ScrapeStatus::Failed;
                }
            }
        } else {
            game_data.metadata.status = ScrapeStatus::Failed;
        }

        send_progress(
            progress_tx,
            progress,
            "Added metadata to gamelist".to_string(),
            Some(game_data.clone()),
        );
        return true;
    }

    // All scrapers failed - mark as failed
    game_data.metadata.status = ScrapeStatus::Failed;

    if tried_any && all_not_found {
        send_progress(
            progress_tx,
            progress,
            "Not found in any source".to_string(),
            Some(game_data.clone()),
        );
    } else {
        send_progress(
            progress_tx,
            progress,
            "Failed to scrape".to_string(),
            Some(game_data.clone()),
        );
    }
    false
}

/// Scrape game guides using a guides scraper
pub async fn scrape_game_guides(
    guides_scrapers: &[Box<dyn GuidesScraper>],
    rom: &scanner::RomFile,
    config: &ScrapingConfig,
    game_data: &mut GameData,
    progress: &ScrapingProgress,
    progress_tx: &UnboundedSender<ProgressUpdate>,
    backoff: &mut BackoffState,
) {
    let console_dir = rom.path.parent().unwrap();
    let guides_dir = console_dir
        .join(&config.guides_folder)
        .join(&rom.name_no_extension);

    // Check if guides already exist (skip this check if skip_cache is enabled)
    let guides_exist = !config.skip_cache
        && guides_dir.exists()
        && std::fs::read_dir(&guides_dir)
            .map(|entries| entries.count() > 0)
            .unwrap_or(false);
    if guides_exist {
        let count = std::fs::read_dir(&guides_dir)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        if count > 0 {
            // Load existing game data to get guides count if available
            if let Some(existing) = load_game_data(&config.roms_path, rom) {
                if existing.guides.count.is_some() {
                    game_data.guides.count = existing.guides.count;
                } else {
                    game_data.guides.count = Some(count);
                }
            } else {
                game_data.guides.count = Some(count);
            }

            game_data.guides.status = ScrapeStatus::Skipped;

            send_progress(
                progress_tx,
                progress,
                "Guides already exist, skipping".to_string(),
                Some(game_data.clone()),
            );
            return;
        }
    }

    game_data.guides.status = ScrapeStatus::Searching;
    send_progress(
        progress_tx,
        progress,
        format!("Searching for guides for: {}", rom.name),
        Some(game_data.clone()),
    );

    for guides_scraper in guides_scrapers {
        match guides_scraper
            .search_game_guides(&rom.path, &rom.console)
            .await
        {
            Ok(guide_paths) if !guide_paths.is_empty() => {
                // Success - reset backoff for this scraper
                backoff.reset(guides_scraper.name());

                game_data.guides.status = ScrapeStatus::Success;
                if let Err(e) = std::fs::create_dir_all(&guides_dir) {
                    send_progress(
                        progress_tx,
                        progress,
                        format!("Warning: Failed to create guides directory: {}", e),
                        None,
                    );
                    return;
                }

                let mut downloaded_guides = Vec::new();
                for guide_path in guide_paths.iter() {
                    let guide_filename = guide_path.split('/').next_back().unwrap_or("guide.txt");
                    let guide_dest = guides_dir.join(guide_filename);
                    match guides_scraper.download_guide(guide_path, &guide_dest).await {
                        Ok(_) => {
                            downloaded_guides.push(PathBuf::from(format!(
                                "./{}/{}/{}",
                                config.guides_folder, rom.name_no_extension, guide_filename
                            )));
                        }
                        Err(e) => {
                            send_progress(
                                progress_tx,
                                progress,
                                format!("Failed to download guide: {}", e),
                                None,
                            );
                        }
                    }
                }

                send_progress(
                    progress_tx,
                    progress,
                    format!("Found {} guide(s)", guide_paths.len()),
                    Some(game_data.clone()),
                );
                return;
            }
            Ok(_) => {
                // No guides found - this counts as "not found"
                game_data.guides.status = ScrapeStatus::Failed;
                send_progress(
                    progress_tx,
                    progress,
                    "No guides found".to_string(),
                    Some(game_data.clone()),
                );
            }
            Err(e) => {
                use crate::scraper::ScraperError;

                // Check if this is a GameNotFound error
                if matches!(e, ScraperError::GameNotFound) {
                    warn!(
                        "Guides scraper {} could not find game: {}",
                        guides_scraper.name(),
                        rom.name
                    );
                } else if matches!(e, ScraperError::RateLimitExceeded) {
                    // Apply exponential backoff immediately
                    warn!(
                        "Guides scraper {} rate limited for game: {}. Applying backoff...",
                        guides_scraper.name(),
                        rom.name
                    );
                    backoff
                        .apply_backoff(guides_scraper.name(), progress_tx, progress)
                        .await;
                } else {
                    warn!(
                        "Guides scraper {} failed for {}: {:?}",
                        guides_scraper.name(),
                        rom.name,
                        e
                    );
                }
                game_data.guides.status = ScrapeStatus::Failed;
                send_progress(
                    progress_tx,
                    progress,
                    format!("Guide search error: {}", e),
                    Some(game_data.clone()),
                );
            }
        }
    }
}
