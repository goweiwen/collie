use std::path::PathBuf;

use super::*;

fn snes_console() -> Console {
    Console {
        name: "SNES".to_owned(),
        patterns: vec!["SNES".to_owned(), "SFC".to_owned()],
        thegamesdb_id: Some(4),
        screenscraper_id: None,
        gamefaqs_archive_id: None,
    }
}

#[test]
fn test_new() {
    let scraper = TheGamesDB::new();
    assert!(scraper.api_key.is_none());
}

#[test]
fn test_with_api_key() {
    let scraper = TheGamesDB::with_api_key("test_key".to_string());
    assert_eq!(scraper.api_key, Some("test_key".to_string()));
}

#[tokio::test]
async fn test_authenticate() {
    let mut scraper = TheGamesDB::new();

    assert!(scraper.api_key.is_none());

    let result = scraper.authenticate("my_api_key", "ignored").await;

    assert!(result.is_ok());
    assert_eq!(scraper.api_key, Some("my_api_key".to_string()));
}

#[tokio::test]
async fn test_search_without_api_key() {
    let scraper = TheGamesDB::new();

    let result = scraper
        .search_game(&PathBuf::from("Super Mario (USA).zip"), &snes_console())
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ScraperError::AuthenticationFailed
    ));
}

#[tokio::test]
async fn test_get_api_key_when_present() {
    let scraper = TheGamesDB::with_api_key("test_key".to_string());

    let result = scraper.get_api_key();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test_key");
}

#[tokio::test]
async fn test_get_api_key_when_missing() {
    let scraper = TheGamesDB::new();

    let result = scraper.get_api_key();

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ScraperError::AuthenticationFailed
    ));
}

// Integration tests - these hit the real API and are ignored by default
#[tokio::test]
#[ignore]
async fn test_search_game_integration() {
    // Get API key from environment variable
    let api_key = std::env::var("THEGAMESDB_API_KEY")
        .expect("Set THEGAMESDB_API_KEY environment variable to run this test");

    let scraper = TheGamesDB::with_api_key(api_key);

    // Search for Super Mario World on SNES (platform 6)
    let result = scraper
        .search_game(
            &PathBuf::from("Super Mario World (USA).zip"),
            &snes_console(),
        )
        .await;

    match result {
        Ok(game) => {
            println!("  - {}", game.name);
            if let Some(ref url) = game.image_url {
                println!("    Image: {}", url);
            }
            assert!(game.name.contains("Mario"));
        }
        Err(e) => {
            panic!("Search failed: {:?}", e);
        }
    }
}
