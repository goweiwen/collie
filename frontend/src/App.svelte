<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import ConfigSection from './lib/components/ConfigSection.svelte';
  import ProgressSection from './lib/components/ProgressSection.svelte';
  import type { GameData, ProgressUpdate } from './lib/types';
  import {
    romsPath,
    boxArtWidth,
    boxArtType,
    useScreenScraper,
    useTheGamesDB,
    ssUsername,
    ssPassword,
    tgdbApiKey,
    useGameFAQs,
    skipCache,
  } from './lib/stores';
  import {
    loadState,
    loadGames,
    loadGamesBatch,
    saveSettings as apiSaveSettings,
    startScraping as apiStartScraping,
    stopScraping as apiStopScraping,
    connectToProgressStream,
  } from './lib/api';

  let scraping = $state(false);
  let cancelled = $state(false);
  let progress = $state(0);
  let totalGames = $state(0);
  let successCount = $state(0);
  let failCount = $state(0);
  let skipCount = $state(0);
  let games = $state<SvelteMap<string, GameData>>(new SvelteMap());
  let currentMessage = $state('');
  let eventSource = $state<EventSource | null>(null);
  let totalGamesCount = $state(0);
  let loadingMore = $state(false);

  // Load state from backend on mount
  async function loadAppState() {
    try {
      const state = await loadState($romsPath);

      scraping = state.scraping;
      progress = state.progress;
      totalGames = state.total_games;
      successCount = state.success_count;
      failCount = state.fail_count;
      skipCount = state.skip_count;
      currentMessage = state.current_message;

      // Load initial batch of games
      await loadMoreGames();

      // If scraping is active, connect to progress stream
      if (state.scraping) {
        connectStream();
      }
    } catch (error) {
      console.error('Error loading state:', error);
    }
  }

  // Load more games (paginated)
  async function loadMoreGames() {
    if (loadingMore) return; // Prevent duplicate requests

    try {
      loadingMore = true;
      const offset = games.size;
      const limit = 10;

      const response = await loadGames(offset, limit);
      totalGamesCount = response.total;

      if (response.games.length > 0) {
        console.log(`Loading ${response.games.length} games (${offset + 1}-${offset + response.games.length} of ${response.total})...`);

        const gameDataList = await loadGamesBatch(response.games);

        // Add new games to the map
        for (const gameData of gameDataList) {
          games.set(gameData.rom_name, gameData);
        }

        console.log(`Loaded ${games.size} of ${response.total} games`);
      }
    } catch (error) {
      console.error('Error loading games:', error);
    } finally {
      loadingMore = false;
    }
  }

  function connectStream() {
    if (eventSource) return; // Already connected

    const handleMessage = (update: ProgressUpdate) => {
      progress = update.completed;
      totalGames = update.total;
      successCount = update.success_count;
      failCount = update.fail_count;
      skipCount = update.skip_count;
      currentMessage = update.message;

      // Update game data if provided
      if (update.game_update) {
        const gameData: GameData = update.game_update;
        console.log('Adding/updating game:', gameData.rom_name, gameData);
        games.set(gameData.rom_name, gameData);
        console.log('Total games in map:', games.size);
      }

      // Detect when scraping finishes (complete or cancelled)
      if (update.message.toLowerCase().includes('scraping cancelled')) {
        cancelled = true;
        scraping = false;
      } else if (update.message.toLowerCase().includes('scraping complete')) {
        cancelled = false;
        scraping = false;
      }
    };

    const handleError = () => {
      if (eventSource) {
        eventSource = null;
      }
      // Don't reset cancelled here - preserve the state
      scraping = false;
    };

    eventSource = connectToProgressStream(handleMessage, handleError);
  }

  // Load state on mount
  loadAppState();

  // Refetch state and games when romsPath changes
  let previousRomsPath = $romsPath;
  $effect(() => {
    const currentPath = $romsPath;
    if (currentPath !== previousRomsPath) {
      previousRomsPath = currentPath;
      // Don't refetch while scraping
      if (!scraping) {
        // Clear existing games before fetching new ones
        games = new SvelteMap();
        progress = 0;
        totalGames = 0;
        successCount = 0;
        failCount = 0;
        skipCount = 0;
        currentMessage = '';

        loadAppState();
      }
    }
  });

  async function startScraping() {
    if (!$useScreenScraper && !$useTheGamesDB && !$useGameFAQs) {
      alert('Please select at least one scraper backend (metadata or guides)');
      return;
    }

    // Validate credentials
    if ($useScreenScraper && $ssUsername && !$ssPassword) {
      alert('Please provide a password for ScreenScraper');
      return;
    }

    if ($useTheGamesDB && !$tgdbApiKey) {
      alert('Please provide an API key for TheGamesDB');
      return;
    }

    // Save settings first
    try {
      await apiSaveSettings({ romsPath: $romsPath });
    } catch (error) {
      console.error('Error saving settings:', error);
    }

    // Clear previous results when starting a new scraping session
    scraping = true;
    cancelled = false;
    progress = 0;
    totalGames = 0;
    games = new SvelteMap();
    currentMessage = '';

    const config = {
      romsPath: $romsPath,
      boxArtWidth: $boxArtWidth,
      skipCache: $skipCache,
      metadataBackends: {
        screenscraper: $useScreenScraper ? {
          username: $ssUsername || null,
          password: $ssPassword || null,
          boxArtType: $boxArtType
        } : null,
        thegamesdb: $useTheGamesDB ? {
          apiKey: $tgdbApiKey
        } : null
      },
      guideBackends: {
        gamefaqs: $useGameFAQs ? true : null
      }
    };

    try {
      const result = await apiStartScraping(config);

      if (result.success) {
        console.log('Scraping started successfully');
        connectStream();
      } else {
        alert('Failed to start scraping: ' + result.message);
        scraping = false;
      }
    } catch (error) {
      alert('Error starting scraping: ' + error);
      scraping = false;
    }
  }

  async function stopScraping() {
    try {
      const result = await apiStopScraping();

      if (result.success) {
        console.log('Scraping stopped successfully');
      } else {
        console.error('Failed to stop scraping:', result.message);
      }
    } catch (error) {
      console.error('Error stopping scraping:', error);
    }

    // Close event source but keep the data visible
    if (eventSource) {
      eventSource.close();
      eventSource = null;
    }

    // Update scraping state but DON'T clear games, progress, or currentMessage
    scraping = false;
    cancelled = true;
  }
</script>

<main>
  <header class="app-header">
    <h1>Collie</h1>
    <p class="tagline">Box-art and metadata scraper for retro games</p>
  </header>

  <div class="app-layout">
    <aside class="control-panel">
      <ConfigSection {scraping} />

      <div class="action-buttons">
        {#if !scraping}
          <button onclick={startScraping} class="start-btn">
            Start Scraping
          </button>
        {:else}
          <button onclick={stopScraping} class="danger stop-btn">
            Stop Scraping
          </button>
        {/if}
      </div>
    </aside>

    <section class="results-panel">
      <ProgressSection
        {scraping}
        {cancelled}
        {progress}
        {totalGames}
        {successCount}
        {failCount}
        {skipCount}
        {currentMessage}
        {games}
        {totalGamesCount}
        {loadingMore}
        {loadMoreGames}
        useGameFAQs={$useGameFAQs}
      />
    </section>
  </div>
</main>

<style>
  main {
    text-align: left;
  }

  .app-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .tagline {
    color: var(--text-muted);
    font-size: 1.1rem;
    margin-top: 0.5rem;
  }

  .app-layout {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .control-panel {
    background: var(--bg-card);
    border-radius: 16px;
    padding: 1.5rem;
    border: 1px solid var(--border-color);
    backdrop-filter: blur(10px);
  }

  .action-buttons {
    margin-top: 1.5rem;
  }

  .start-btn, .stop-btn {
    width: 100%;
    padding: 1rem 2rem;
    font-size: 1.1rem;
  }

  .results-panel {
    flex: 1;
    min-height: 400px;
  }

  /* Widescreen layout - side by side */
  @media (min-width: 1024px) {
    .app-layout {
      flex-direction: row;
      align-items: flex-start;
    }

    .control-panel {
      width: 380px;
      flex-shrink: 0;
    }

    .results-panel {
      flex: 1;
      min-width: 0;
    }
  }

  /* Extra wide screens */
  @media (min-width: 1400px) {
    .control-panel {
      width: 420px;
    }
  }
</style>
