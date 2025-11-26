use backoff::ExponentialBackoff;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;

use crate::progress::{ProgressUpdate, ScrapingProgress, send_progress};

/// Tracks exponential backoff state for rate-limited scrapers
pub struct BackoffState {
    /// Maps scraper name to their backoff state
    scrapers: HashMap<String, ExponentialBackoff>,
}

impl BackoffState {
    pub fn new() -> Self {
        Self {
            scrapers: HashMap::new(),
        }
    }

    /// Get or create a backoff for a scraper
    fn get_backoff(&mut self, scraper_name: &str) -> &mut ExponentialBackoff {
        self.scrapers
            .entry(scraper_name.to_string())
            .or_insert_with(|| {
                ExponentialBackoff {
                    current_interval: Duration::from_secs(1),
                    initial_interval: Duration::from_secs(1),
                    randomization_factor: 0.0,
                    multiplier: 2.0,
                    max_interval: Duration::from_secs(300), // 5 minutes max
                    max_elapsed_time: None,                 // Never give up
                    ..Default::default()
                }
            })
    }

    /// Get the next backoff duration for a scraper and apply it
    pub async fn apply_backoff(
        &mut self,
        scraper_name: &str,
        progress_tx: &UnboundedSender<ProgressUpdate>,
        progress: &ScrapingProgress,
    ) {
        use backoff::backoff::Backoff;

        let backoff = self.get_backoff(scraper_name);
        if let Some(duration) = backoff.next_backoff() {
            let secs = duration.as_secs();
            warn!(
                "Pausing scrape for {}s due to rate limit from {}",
                secs, scraper_name
            );
            send_progress(
                progress_tx,
                progress,
                format!("Pausing for {}s (rate limited by {})", secs, scraper_name),
                None,
            );
            tokio::time::sleep(duration).await;
        }
    }

    /// Reset backoff for a scraper after successful request
    pub fn reset(&mut self, scraper_name: &str) {
        self.scrapers.remove(scraper_name);
    }
}

impl Default for BackoffState {
    fn default() -> Self {
        Self::new()
    }
}
