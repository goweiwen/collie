mod server;

use axum::{Router, routing::{get, post}};
use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use server::api::directories::list_directories;
use server::api::games::{get_game_by_rom_name, get_games};
use server::api::images::serve_image;
use server::api::scrape::{get_state, progress_stream, save_settings, start_scraping, stop_scraping};
use server::state::{AppState, load_state};
use server::static_files::static_handler;

/// Broadcast channel buffer size for progress updates
const PROGRESS_CHANNEL_SIZE: usize = 100;

#[derive(Parser, Debug)]
#[command(name = "collie")]
#[command(about = "Box-art and metadata scraper for retro games", long_about = None)]
struct Args {
    /// Address to bind the web server to
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: IpAddr,

    /// Port to run the web server on
    #[arg(short, long, default_value_t = 2435)]
    port: u16,

    /// Don't launch the web interface in the default browser
    #[arg(long)]
    no_launch: bool,

    /// Clear all cached data and start fresh (removes .collie/cache, games, state)
    #[arg(long)]
    no_cache: bool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    // Set RUST_LOG env var to control log level, e.g., RUST_LOG=debug
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let args = Args::parse();

    println!("Collie - ROM Metadata Scraper");
    println!("Web server port: {}", args.port);

    let roms_path = std::env::current_dir().expect("Failed to get current directory");

    // Delete cache and stored data if --no-cache flag is set
    if args.no_cache {
        use collie::cache::ScrapeCache;
        let collie_dir = roms_path.join(".collie");

        // Clear the failure markers cache
        let cache = ScrapeCache::new(&roms_path);
        if let Err(e) = cache.clear_all() {
            eprintln!("Warning: Failed to clear cache: {}", e);
        }

        // Clear stored game data
        let games_dir = collie_dir.join("games");
        if games_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&games_dir) {
                eprintln!("Warning: Failed to clear games directory: {}", e);
            }
        }

        // Clear games index
        let games_index = collie_dir.join("games.txt");
        if games_index.exists() {
            if let Err(e) = std::fs::remove_file(&games_index) {
                eprintln!("Warning: Failed to clear games index: {}", e);
            }
        }

        // Clear scraping state
        let state_file = collie_dir.join("state.json");
        if state_file.exists() {
            if let Err(e) = std::fs::remove_file(&state_file) {
                eprintln!("Warning: Failed to clear state file: {}", e);
            }
        }

        // Clear crawled file
        let crawled_file = collie_dir.join("crawled");
        if crawled_file.exists() {
            if let Err(e) = std::fs::remove_file(&crawled_file) {
                eprintln!("Warning: Failed to clear crawled file: {}", e);
            }
        }

        println!("Cache and stored data cleared");
    }

    // Load scraping state from previous sessions
    let initial_state = load_state(&roms_path);
    if initial_state.progress > 0 {
        println!(
            "Loaded state: {}/{} games processed (Success: {}, Skipped: {}, Failed: {})",
            initial_state.progress,
            initial_state.total_games,
            initial_state.success_count,
            initial_state.skip_count,
            initial_state.fail_count
        );
    }

    let (progress_tx, _) = broadcast::channel(PROGRESS_CHANNEL_SIZE);

    let state = AppState {
        scraping: Arc::new(Mutex::new(false)),
        cancel_token: Arc::new(Mutex::new(None)),
        progress_tx,
        roms_path: Arc::new(std::sync::Mutex::new(roms_path)),
        scraping_state: Arc::new(Mutex::new(initial_state)),
    };

    let app = Router::new()
        .route("/api/settings", post(save_settings))
        .route("/api/scrape", post(start_scraping))
        .route("/api/stop", post(stop_scraping))
        .route("/api/state", post(get_state))
        .route("/api/directories", post(list_directories))
        .route("/api/games", get(get_games))
        .route("/api/games/{rom_name}", get(get_game_by_rom_name))
        .route("/api/progress", get(progress_stream))
        .route("/api/images/{*path}", get(serve_image))
        .with_state(state)
        .fallback(static_handler)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::DEBUG)),
        );

    let addr = SocketAddr::new(args.bind, args.port);
    let url = format!("http://localhost:{}", args.port);

    println!("\nListening on {}", addr);
    println!("Open this URL in your browser to configure the scraper.\n");

    // Launch browser by default unless --no-launch is specified
    if !args.no_launch {
        println!("Launching web browser...");
        if let Err(e) = webbrowser::open(&url) {
            eprintln!("Failed to open browser: {}", e);
            println!("Please open {} manually.", url);
        }
    }

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
