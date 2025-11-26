<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import type { GameData } from '../types';
  import GameCard from './GameCard.svelte';

  let {
    scraping = false,
    cancelled = false,
    progress = 0,
    totalGames = 0,
    successCount = 0,
    failCount = 0,
    skipCount = 0,
    currentMessage = '',
    games = new SvelteMap<string, GameData>(),
    totalGamesCount = 0,
    loadingMore = false,
    loadMoreGames = () => {},
    useGameFAQs = false
  }: {
    scraping?: boolean;
    cancelled?: boolean;
    progress?: number;
    totalGames?: number;
    successCount?: number;
    failCount?: number;
    skipCount?: number;
    currentMessage?: string;
    games?: SvelteMap<string, GameData>;
    totalGamesCount?: number;
    loadingMore?: boolean;
    loadMoreGames?: () => Promise<void>;
    useGameFAQs?: boolean;
  } = $props();

  const percentage = $derived(totalGames > 0 ? Math.round((progress / totalGames) * 100) : 0);

  // Infinite scroll
  let loadMoreRef = $state<HTMLDivElement | null>(null);

  // Games are already in newest-first order from the backend
  const allGames = $derived(Array.from(games.values()));
  const hasMore = $derived(games.size < totalGamesCount);

  $effect(() => {
    if (!loadMoreRef) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !loadingMore) {
          loadMoreGames();
        }
      },
      { rootMargin: '200px' }
    );

    observer.observe(loadMoreRef);

    return () => observer.disconnect();
  });
</script>

<div class="progress-section">
  {#if totalGames > 0 || scraping}
    <div class="progress-sticky">
      <div class="progress-header">
        <h2>{scraping ? 'Scraping...' : cancelled ? 'Cancelled!' : 'Complete!'}</h2>
        <span class="progress-count">{progress} / {totalGames}</span>
      </div>

      <div class="progress-bar-container">
        <progress value={progress} max={totalGames}></progress>
        <span class="percentage">{percentage}%</span>
      </div>

      {#if currentMessage}
        <div class="current-message">
          <span class="pulse"></span>
          {currentMessage}
        </div>
      {/if}
    </div>

    <div class="stats-row">
      <div class="stat success">
        <span class="stat-value">{successCount}</span>
        <span class="stat-label">Success</span>
      </div>
      <div class="stat skipped">
        <span class="stat-value">{skipCount}</span>
        <span class="stat-label">Skipped</span>
      </div>
      <div class="stat failed">
        <span class="stat-value">{failCount}</span>
        <span class="stat-label">Failed</span>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
          <line x1="8" y1="21" x2="16" y2="21"></line>
          <line x1="12" y1="17" x2="12" y2="21"></line>
        </svg>
      </div>
      <h3>Ready to scrape!</h3>
      <p>Set your ROMs folder, pick your sources, and hit Start</p>
    </div>
  {/if}

  {#if games.size > 0}
    <div class="games-section">
      <h3 class="games-header">
        Recently Scraped Games
        <span class="count">{totalGamesCount > 0 ? totalGamesCount : games.size}</span>
      </h3>
      <div class="game-cards">
        {#each allGames as game (game.rom_name)}
          <GameCard {game} {useGameFAQs} />
        {/each}
      </div>
      {#if hasMore}
        <div class="load-more" bind:this={loadMoreRef}>
          <span class="loading-spinner"></span>
          Loading more...
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .progress-sticky {
    position: sticky;
    top: 1rem;
    z-index: 10;
    background: var(--bg-card);
    padding: 1rem;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  @media (min-width: 1024px) {
    .progress-sticky {
      position: static;
    }
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .progress-header h2 {
    font-size: 1.5rem;
    margin: 0;
  }

  .progress-count {
    font-size: 1rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .progress-bar-container {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .progress-bar-container progress {
    flex: 1;
  }

  .percentage {
    min-width: 50px;
    text-align: right;
    font-weight: 700;
    font-size: 1.1rem;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .stats-row {
    display: flex;
    gap: 1rem;
  }

  .stat {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
    background: var(--bg-card);
    border-radius: 12px;
    border: 2px solid var(--border-color);
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .stat.success {
    border-color: var(--success);
  }

  .stat.success .stat-value {
    color: var(--success);
  }

  .stat.skipped {
    border-color: var(--warning);
  }

  .stat.skipped .stat-value {
    color: var(--warning);
  }

  .stat.failed {
    border-color: var(--danger);
  }

  .stat.failed .stat-value {
    color: var(--danger);
  }

  .current-message {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: rgba(116, 185, 255, 0.1);
    border: 1px solid rgba(116, 185, 255, 0.3);
    border-radius: 10px;
    font-size: 0.9rem;
    color: var(--info);
  }

  .pulse {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--info);
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(1.2); }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 4rem 2rem;
    background: var(--bg-card);
    border-radius: 16px;
    border: 2px dashed var(--border-color);
  }

  .empty-icon {
    color: var(--text-muted);
    margin-bottom: 1rem;
    opacity: 0.6;
  }

  .empty-state h3 {
    font-size: 1.25rem;
    margin: 0 0 0.5rem 0;
  }

  .empty-state p {
    color: var(--text-muted);
    margin: 0;
    font-size: 0.95rem;
  }

  .games-section {
    margin-top: 0.5rem;
  }

  .games-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
    font-size: 1.1rem;
  }

  .count {
    background: var(--primary);
    color: white;
    padding: 0.15rem 0.6rem;
    border-radius: 20px;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .game-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
  }

  .load-more {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1.5rem;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .loading-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-color);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @media (prefers-color-scheme: light) {
    .current-message {
      background: rgba(116, 185, 255, 0.08);
    }
  }
</style>
