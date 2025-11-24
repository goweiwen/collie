<script lang="ts">
  import type { GameData } from '../types';
  import { boxArtWidth, boxArtType } from '../stores';

  let { game, useGameFAQs = false }: { game: GameData; useGameFAQs?: boolean } = $props();

  const combinedStatus = $derived(game.status || game.metadata.status);

  const statusConfig = $derived({
    success: { color: 'var(--success)', label: 'Found', icon: '✓' },
    failed: { color: 'var(--danger)', label: 'Failed', icon: '✗' },
    skipped: { color: 'var(--warning)', label: 'Skipped', icon: '→' },
    searching: { color: 'var(--info)', label: 'Searching', icon: '...' },
    pending: { color: 'var(--text-muted)', label: 'Pending', icon: '○' }
  }[combinedStatus] || { color: 'var(--text-muted)', label: combinedStatus, icon: '?' });
</script>

<article class="game-card status-{combinedStatus}">
  <header class="card-header">
    <h4 class="rom-name" title={game.rom_name}>{game.rom_name}</h4>
    <span class="status-badge" style="background: {statusConfig.color}">
      {statusConfig.icon}
    </span>
  </header>

  <div class="game-info">

    {#if game.metadata.image_path}
      <div class="game-image">
        <img
          src="{game.metadata.image_path}?type={$boxArtType}"
          alt={game.metadata.name || game.rom_name}
          width={$boxArtWidth}
          loading="lazy"
        />
      </div>
    {/if}

    {#if game.metadata.name}
    <h5 class="game-title">{game.metadata.name}</h5>
    {/if}

    <div class="metadata">
      {#if game.metadata.developer}
        <span class="meta-item">
          <span class="meta-label">Dev</span>
          <span class="meta-value">{game.metadata.developer}</span>
        </span>
      {/if}
      {#if game.metadata.publisher}
        <span class="meta-item">
          <span class="meta-label">Pub</span>
          <span class="meta-value">{game.metadata.publisher}</span>
        </span>
      {/if}
      {#if game.metadata.release_date}
        <span class="meta-item">
          <span class="meta-label">Rel</span>
          <span class="meta-value">{game.metadata.release_date}</span>
        </span>
      {/if}
      {#if game.metadata.rating}
        <span class="meta-item">
          <span class="meta-label">Rat</span>
          <span class="meta-value">{game.metadata.rating}</span>
        </span>
      {/if}
    </div>

    <div class="tags">
      {#if game.metadata.genre}
        {#each game.metadata.genre.split(',') as genre}
          <span class="tag">{genre.trim()}</span>
        {/each}
      {/if}
    </div>

    {#if useGameFAQs}
      <div class="guides-section">
        {#if game.guides.status === 'searching'}
          <div class="guides-badge guides-searching">
            <span class="guides-icon">📖</span>
            <span>Searching guides...</span>
          </div>
        {:else if game.guides.count !== undefined && game.guides.count > 0}
          <div class="guides-badge guides-success">
            <span class="guides-icon">📖</span>
            <span>{game.guides.count} guide{game.guides.count > 1 ? 's' : ''}</span>
          </div>
        {:else if game.guides.status === 'failed'}
          <div class="guides-badge guides-failed">
            <span class="guides-icon">📖</span>
            <span>No guides found</span>
          </div>
        {:else if game.guides.status === 'skipped'}
          <div class="guides-badge guides-skipped">
            <span class="guides-icon">📖</span>
            <span>Guides cached</span>
          </div>
        {/if}
      </div>
    {/if}

    {#if game.metadata.status === 'skipped'}
      <p class="status-note">Already scraped</p>
    {/if}
  </div>
</article>

<style>
  .game-card {
    background: var(--bg-card);
    border: 2px solid var(--border-color);
    border-radius: 14px;
    overflow: hidden;
    transition: all 0.25s ease;
  }

  .game-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
  }

  .game-card.status-success {
    border-color: var(--success);
  }

  .game-card.status-failed {
    border-color: var(--danger);
  }

  .game-card.status-skipped {
    border-color: var(--warning);
  }

  .game-card.status-searching {
    border-color: var(--info);
    animation: searchPulse 2s ease-in-out infinite;
  }

  @keyframes searchPulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(116, 185, 255, 0.4); }
    50% { box-shadow: 0 0 0 8px rgba(116, 185, 255, 0); }
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.04);
  }

  .rom-name {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    margin-right: 0.5rem;
  }

  .status-badge {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 700;
    color: white;
    flex-shrink: 0;
  }

  .game-info {
    padding: 1rem;
  }

  .game-image {
    margin: 0 0 0.75rem 0;
    width: 100%;
  }

  .game-image img {
    display: block;
    width: 100%;
    height: auto;
    max-height: 300px;
    object-fit: contain;
    border-radius: 14px;
  }

  .game-title {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    font-weight: 700;
    line-height: 1.3;
  }

  .metadata {
    margin: 0 0 0.75rem 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    font-size: 0.75rem;
  }

  .meta-label {
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
    font-size: 0.65rem;
    letter-spacing: 0.5px;
  }

  .meta-value {
    font-weight: 500;
  }

  .tag {
    padding: 0.2rem 0.5rem;
    margin-right: 0.5rem;
    background: rgba(0, 0, 0, 0.06);
    border-radius: 6px;
    font-size: 0.7rem;
    font-weight: 500;
  }

  .rating {
    background: rgba(253, 203, 110, 0.2);
    color: var(--warning);
  }

  .guides-section {
    margin-top: 0.75rem;
  }

  .guides-badge {
    padding: 0.4rem 0.75rem;
    border-radius: 8px;
    font-size: 0.8rem;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .guides-icon {
    font-size: 0.9rem;
  }

  .guides-success {
    background: rgba(108, 92, 231, 0.15);
    color: var(--primary);
  }

  .guides-searching {
    background: rgba(116, 185, 255, 0.15);
    color: var(--info);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .guides-failed {
    background: rgba(255, 107, 107, 0.1);
    color: var(--text-muted);
  }

  .guides-skipped {
    background: rgba(253, 203, 110, 0.15);
    color: var(--warning);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  .status-note {
    margin: 0.75rem 0 0 0;
    font-size: 0.75rem;
    color: var(--text-muted);
    font-style: italic;
  }
</style>
