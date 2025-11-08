pub mod backoff;
pub mod cache;
pub mod console;
pub mod gamelist;
pub mod image;
pub mod progress;
pub mod scanner;
pub mod scraper;
pub mod scraping;
pub mod storage;

use backoff::BackoffState;
use console::ConsolesConfig;
use scanner::RomScanner;
use scraper::{GuidesScraper, MetadataScraper};
use storage::{append_crawled_path, append_scraped_index, save_game_data};
use tracing::info;

// Re-export commonly used types
pub use progress::{GameData, GameGuides, GameMetadata, ProgressUpdate, ScrapeStatus, ScrapingProgress, send_progress};
pub use scraping::{ScrapingConfig, scrape_game_guides, scrape_game_metadata};

pub async fn scrape(
    metadata_scrapers: Vec<Box<dyn MetadataScraper>>,
    guides_scrapers: Vec<Box<dyn GuidesScraper>>,
    config: ScrapingConfig,
    cancel_token: tokio_util::sync::CancellationToken,
    progress_tx: tokio::sync::mpsc::UnboundedSender<ProgressUpdate>,
) -> Result<ScrapingProgress, Box<dyn std::error::Error + Send + Sync>> {
    // Initialize backoff state for rate limiting
    let mut backoff = BackoffState::new();

    // Load console configuration
    let consoles_config = ConsolesConfig::from_embedded()
        .map_err(|e| format!("Failed to load console config: {}", e))?;

    // Scan for ROMs
    let scanner = RomScanner::new(consoles_config);
    let rom_files = scanner
        .scan_directory(&config.roms_path)
        .map_err(|e| format!("Failed to scan ROMs: {}", e))?;

    let total = rom_files.len();
    let mut progress = progress::ScrapingProgress {
        total,
        completed: 0,
        current_rom: None,
        success_count: 0,
        fail_count: 0,
        skip_count: 0,
    };

    info!("\n========================================");
    info!("Starting ROM Scraping");
    info!("========================================");
    info!("Found {} ROMs to process\n", total);

    progress::send_progress(
        &progress_tx,
        &progress,
        format!("Found {} ROMs to process", total),
        None,
    );

    let mut cancelled = false;
    for rom in rom_files {
        // Check if cancellation was requested
        if cancel_token.is_cancelled() {
            info!("\n⚠ Scraping cancelled by user");
            progress::send_progress(
                &progress_tx,
                &progress,
                "Scraping cancelled by user".to_string(),
                None,
            );
            cancelled = true;
            break;
        }

        progress.current_rom = Some(rom.name.clone());

        info!(
            "\n[{}/{}] Processing: {} ({})",
            progress.completed + 1,
            progress.total,
            rom.name,
            rom.console.name
        );

        // Create initial game data entry
        let mut game_data = progress::GameData {
            rom_name: rom.name.clone(),
            metadata: progress::GameMetadata {
                status: progress::ScrapeStatus::Pending,
                name: None,
                developer: None,
                publisher: None,
                genre: None,
                release_date: None,
                rating: None,
                image_path: None,
                error_message: None,
            },
            guides: progress::GameGuides {
                status: progress::ScrapeStatus::Pending,
                count: None,
            },
        };

        progress::send_progress(
            &progress_tx,
            &progress,
            format!("Processing: {} ({})", rom.name, rom.console.name),
            Some(game_data.clone()),
        );

        scraping::scrape_game_metadata(
            &metadata_scrapers,
            &rom,
            &config,
            &mut game_data,
            &progress,
            &progress_tx,
            &mut backoff,
        )
        .await;

        let status = if !guides_scrapers.is_empty() {
            scraping::scrape_game_guides(
                &guides_scrapers,
                &rom,
                &config,
                &mut game_data,
                &progress,
                &progress_tx,
                &mut backoff,
            )
            .await;

            game_data.metadata.status.merge(game_data.guides.status)
        } else {
            game_data.metadata.status
        };

        if status == progress::ScrapeStatus::Success {
            progress.success_count += 1;
        } else if status == progress::ScrapeStatus::Skipped {
            progress.skip_count += 1;
        } else {
            progress.fail_count += 1;
        }
        progress.completed += 1;

        // Save game data to .collie/games/<path>.json
        if let Err(e) = save_game_data(&config.roms_path, &rom, &game_data) {
            tracing::warn!("Failed to save game data for {}: {}", rom.name, e);
        }

        // Append game entry to .collie/scraped index
        if let Err(e) = append_scraped_index(&config.roms_path, &rom, &game_data) {
            tracing::warn!("Failed to append scraped index for {}: {}", rom.name, e);
        }

        // Append crawled path to .collie/crawled
        if let Err(e) = append_crawled_path(&config.roms_path, &rom) {
            tracing::warn!("Failed to append crawled path for {}: {}", rom.name, e);
        }
    }

    // Cache is automatically persisted to filesystem via marker files
    // No need to save anything here

    // Save all gamelists
    progress.current_rom = None;

    let status_word = if cancelled { "Cancelled" } else { "Complete" };
    info!("\n========================================");
    info!("Scraping {}!", status_word);
    info!("========================================");
    info!("Total ROMs:     {}", progress.total);
    info!("Success:        {}", progress.success_count);
    info!("Skipped:        {}", progress.skip_count);
    info!("Failed:         {}", progress.fail_count);
    info!("========================================\n");

    progress::send_progress(
        &progress_tx,
        &progress,
        format!(
            "Scraping {}! Total: {}, Success: {}, Skipped: {}, Failed: {}",
            status_word.to_lowercase(),
            progress.total,
            progress.success_count,
            progress.skip_count,
            progress.fail_count
        ),
        None,
    );

    Ok(progress)
}
