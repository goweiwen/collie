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
fn test_build_api_url_without_auth() {
    let scraper = ScreenScraper::new();
    let url = scraper.build_api_url("jeuInfos.php", &[("systemeid", "4"), ("romnom", "test")]);

    assert!(url.contains("devid="));
    assert!(url.contains("devpassword="));
    assert!(url.contains("softname=collie"));
    assert!(url.contains("output=json"));
    assert!(url.contains("systemeid=4"));
    assert!(url.contains("romnom=test"));
    assert!(!url.contains("ssid="));
    assert!(!url.contains("sspassword="));
}

#[test]
fn test_build_api_url_with_auth() {
    let mut scraper = ScreenScraper::new();
    scraper.username = Some("testuser".to_string());
    scraper.password = Some("testpass".to_string());

    let url = scraper.build_api_url("jeuInfos.php", &[("systemeid", "4")]);

    assert!(url.contains("ssid=testuser"));
    assert!(url.contains("sspassword=testpass"));
}

#[test]
fn test_parse_game_metadata_full() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "response": {
            "jeu": {
                "id": "12345",
                "noms": [{"text": "Super Mario World"}],
                "synopsis": [{"text": "A classic platformer game"}],
                "dates": [{"text": "1990-11-21"}],
                "developpeur": {"text": "Nintendo EAD"},
                "editeur": {"text": "Nintendo"},
                "genres": [{"noms": [{"text": "Platform"}]}],
                "joueurs": {"text": "1-2"},
                "classifications": [{"text": "4.5"}]
            }
        }
    });

    let metadata = scraper.parse_game_metadata(&json).unwrap();

    assert_eq!(metadata.name, "Super Mario World");
    assert_eq!(
        metadata.description,
        Some("A classic platformer game".to_string())
    );
    assert_eq!(metadata.release_date, Some("1990-11-21".to_string()));
    assert_eq!(metadata.developer, Some("Nintendo EAD".to_string()));
    assert_eq!(metadata.publisher, Some("Nintendo".to_string()));
    assert_eq!(metadata.genre, Some("Platform".to_string()));
    assert_eq!(metadata.players, Some("1-2".to_string()));
    assert_eq!(metadata.rating, Some(4.5));
}

#[test]
fn test_parse_game_metadata_minimal() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "jeu": {
            "noms": [{"text": "Test Game"}]
        }
    });

    let metadata = scraper.parse_game_metadata(&json).unwrap();

    assert_eq!(metadata.name, "Test Game");
    assert_eq!(metadata.description, None);
    assert_eq!(metadata.release_date, None);
    assert_eq!(metadata.developer, None);
    assert_eq!(metadata.publisher, None);
    assert_eq!(metadata.genre, None);
    assert_eq!(metadata.players, None);
    assert_eq!(metadata.rating, None);
}

#[test]
fn test_parse_game_metadata_missing_jeu() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "response": {}
    });

    let result = scraper.parse_game_metadata(&json);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ScraperError::ParseError(_)));
}

#[test]
fn test_get_media_url_exact_region_match() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "response": {
            "jeu": {
                "medias": [
                    {
                        "type": "box-2D",
                        "region": "us",
                        "url": "https://example.com/us.jpg"
                    },
                    {
                        "type": "box-2D",
                        "region": "eu",
                        "url": "https://example.com/eu.jpg"
                    },
                    {
                        "type": "box-2D",
                        "region": "jp",
                        "url": "https://example.com/jp.jpg"
                    }
                ]
            }
        }
    });

    let region_preferences = vec!["eu", "us", "jp"];
    let url = scraper.get_media_url(&json, "box-2D", &region_preferences);

    assert_eq!(url, Some("https://example.com/eu.jpg".to_string()));
}

#[test]
fn test_get_media_url_fallback_region() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "jeu": {
            "medias": [
                {
                    "type": "box-2D",
                    "region": "jp",
                    "url": "https://example.com/jp.jpg"
                },
                {
                    "type": "box-2D",
                    "region": "us",
                    "url": "https://example.com/us.jpg"
                }
            ]
        }
    });

    // Prefer EU but it doesn't exist, should fall back to US
    let region_preferences = vec!["eu", "us", "jp"];
    let url = scraper.get_media_url(&json, "box-2D", &region_preferences);

    assert_eq!(url, Some("https://example.com/us.jpg".to_string()));
}

#[test]
fn test_get_media_url_wrong_type() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "response": {
            "jeu": {
                "medias": [
                    {
                        "type": "screenshot",
                        "region": "us",
                        "url": "https://example.com/screenshot.jpg"
                    }
                ]
            }
        }
    });

    let region_preferences = vec!["us"];
    let url = scraper.get_media_url(&json, "box-2D", &region_preferences);

    assert_eq!(url, None);
}

#[test]
fn test_get_media_url_no_medias() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "response": {
            "jeu": {}
        }
    });

    let region_preferences = vec!["us"];
    let url = scraper.get_media_url(&json, "box-2D", &region_preferences);

    assert_eq!(url, None);
}

#[test]
fn test_get_media_url_multiple_types_filter() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "response": {
            "jeu": {
                "medias": [
                    {
                        "type": "screenshot",
                        "region": "us",
                        "url": "https://example.com/screenshot.jpg"
                    },
                    {
                        "type": "box-2D",
                        "region": "us",
                        "url": "https://example.com/box.jpg"
                    },
                    {
                        "type": "sstitle",
                        "region": "us",
                        "url": "https://example.com/title.jpg"
                    }
                ]
            }
        }
    });

    let region_preferences = vec!["us"];
    let url = scraper.get_media_url(&json, "box-2D", &region_preferences);

    assert_eq!(url, Some("https://example.com/box.jpg".to_string()));
}

#[test]
fn test_get_media_url_region_priority() {
    let scraper = ScreenScraper::new();
    let json = serde_json::json!({
        "response": {
            "jeu": {
                "medias": [
                    {
                        "type": "box-2D",
                        "region": "jp",
                        "url": "https://example.com/jp.jpg"
                    },
                    {
                        "type": "box-2D",
                        "region": "wor",
                        "url": "https://example.com/wor.jpg"
                    },
                    {
                        "type": "box-2D",
                        "region": "us",
                        "url": "https://example.com/us.jpg"
                    }
                ]
            }
        }
    });

    // First preference is JP, should get that one even though US is also available
    let region_preferences = vec!["jp", "us", "eu"];
    let url = scraper.get_media_url(&json, "box-2D", &region_preferences);

    assert_eq!(url, Some("https://example.com/jp.jpg".to_string()));
}

#[tokio::test]
async fn test_authenticate() {
    let mut scraper = ScreenScraper::new();

    assert!(scraper.username.is_none());
    assert!(scraper.password.is_none());

    let result = scraper.authenticate("testuser", "testpass").await;

    assert!(result.is_ok());
    assert_eq!(scraper.username, Some("testuser".to_string()));
    assert_eq!(scraper.password, Some("testpass".to_string()));
}

#[test]
fn test_screenscraper_new() {
    let scraper = ScreenScraper::new();

    assert!(scraper.username.is_none());
    assert!(scraper.password.is_none());
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --nocapture
async fn test_search_game_integration() {
    let scraper = ScreenScraper::new();

    let result = scraper
        .search_game(
            &PathBuf::from("Super Mario World (USA).zip"),
            &snes_console(),
        )
        .await;

    match result {
        Ok(_game) => {}
        Err(e) => {
            println!("Search failed: {:?}", e);
            // Don't fail test - API might be down or rate limited
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_authenticated_search() {
    let mut scraper = ScreenScraper::new();

    // Replace with real credentials to test
    scraper
        .authenticate("YOUR_USERNAME", "YOUR_PASSWORD")
        .await
        .unwrap();

    let result = scraper
        .search_game(
            &PathBuf::from("Super Mario World (USA).zip"),
            &snes_console(),
        )
        .await;

    match result {
        Ok(_game) => {}
        Err(e) => {
            println!("Search failed: {:?}", e);
        }
    }
}
