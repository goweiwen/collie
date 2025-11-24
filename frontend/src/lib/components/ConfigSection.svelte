<script lang="ts">
  import {
    romsPath,
    boxArtWidth,
    boxArtType,
    boxArtTypes,
    boxArtPreviewUrl,
    previewGames,
    useScreenScraper,
    useTheGamesDB,
    ssUsername,
    ssPassword,
    tgdbApiKey,
    useGameFAQs,
    skipCache,
  } from '../stores';
  import FolderPicker from './FolderPicker.svelte';

  let { scraping = false }: { scraping?: boolean } = $props();

  let activeTab: 'basic' | 'advanced' = $state('basic');
  let settingsExpanded = $state(true);

  // Collapse settings when scraping starts
  $effect(() => {
    if (scraping) {
      settingsExpanded = false;
    }
  });
</script>

<div class="config-section">
  <!-- Essential: ROMs Path -->
  <div class="form-group main-input">
    <label for="roms-path">ROMs Folder</label>
    <FolderPicker bind:value={$romsPath} disabled={scraping} />
  </div>

  <!-- Settings Accordion -->
  <div class="settings-accordion">
    <button
      class="accordion-header"
      onclick={() => settingsExpanded = !settingsExpanded}
      aria-expanded={settingsExpanded}
    >
      <span class="accordion-title">Settings</span>
      <span class="accordion-icon" class:expanded={settingsExpanded}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708l-3 3a.5.5 0 0 1-.708 0l-3-3a.5.5 0 0 1 0-.708z"/>
        </svg>
      </span>
    </button>

    {#if settingsExpanded}
      <div class="accordion-content">
        <!-- Boxart Sources -->
        <div class="scraper-toggles">
          <h3>Boxart Sources</h3>
          <div class="toggle-grid">
            <label class="toggle-card" class:active={$useScreenScraper}>
              <input type="checkbox" bind:checked={$useScreenScraper} disabled={scraping} />
              <span class="toggle-label">ScreenScraper</span>
            </label>
            <label class="toggle-card" class:active={$useTheGamesDB}>
              <input type="checkbox" bind:checked={$useTheGamesDB} disabled={scraping} />
              <span class="toggle-label">TheGamesDB</span>
            </label>
          </div>
        </div>

        <!-- Guides Sources -->
        <div class="scraper-toggles">
          <h3>Guides Sources</h3>
          <div class="toggle-grid">
            <label class="toggle-card" class:active={$useGameFAQs}>
              <input type="checkbox" bind:checked={$useGameFAQs} disabled={scraping} />
              <span class="toggle-label">GameFAQs</span>
            </label>
          </div>
        </div>

        <!-- Tabs for Basic/Advanced -->
        <div class="tabs">
          <button
            class="tab-btn"
            class:active={activeTab === 'basic'}
            onclick={() => activeTab = 'basic'}
          >
            Basic
          </button>
          <button
            class="tab-btn"
            class:active={activeTab === 'advanced'}
            onclick={() => activeTab = 'advanced'}
          >
            Advanced
          </button>
        </div>

        <div class="tab-content">
          {#if activeTab === 'basic'}
            <div class="basic-settings">
              <div class="form-group">
                <label for="box-art-width">Box Art Width</label>
                <div class="slider-group">
                  <input
                    id="box-art-width"
                    type="range"
                    min="100"
                    max="500"
                    step="10"
                    bind:value={$boxArtWidth}
                    disabled={scraping}
                  />
                  <span class="slider-value">{$boxArtWidth}px</span>
                </div>
              </div>

              {#if $useScreenScraper}
                <div class="form-group">
                  <label for="box-art-type">Box Art Type</label>
                  <select
                    id="box-art-type"
                    bind:value={$boxArtType}
                    disabled={scraping}
                  >
                    {#each boxArtTypes as type}
                      <option value={type.value} disabled={type.disabled}>{type.label}</option>
                    {/each}
                  </select>
                  <div class="box-art-preview">
                    {#each previewGames as game}
                      <div class="preview-item">
                        <img
                          src={boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, game.region, 1)}
                          alt={`${game.name} - ${$boxArtType}`}
                          loading="lazy"
                          onerror={(e) => {
                            const img = e.currentTarget as HTMLImageElement;
                            const fallbacks = [
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, game.region, 1),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, game.region, 0),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, 'wor', 1),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, 'wor', 0),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, '', 1),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, '', 0),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, game.region, 1, '1'),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, game.region, 0, '1'),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, 'wor', 1, '1'),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, 'wor', 0, '1'),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, '', 1, '1'),
                              boxArtPreviewUrl($boxArtType, game.platformId, game.gameId, '', 0, '1'),
                            ];
                            const currentIndex = fallbacks.indexOf(img.src);
                            if (currentIndex >= 0 && currentIndex < fallbacks.length - 1) {
                              img.src = fallbacks[currentIndex + 1];
                            }
                          }}
                        />
                        <span class="preview-caption">{game.name}</span>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {:else}
            <div class="advanced-settings">
              {#if $useScreenScraper}
                <div class="credential-section">
                  <h4>ScreenScraper Credentials</h4>
                  <p class="hint">Optional - improves rate limits</p>
                  <div class="form-row">
                    <div class="form-group">
                      <label for="ss-username">Username</label>
                      <input
                        id="ss-username"
                        type="text"
                        bind:value={$ssUsername}
                        disabled={scraping}
                        placeholder="Optional"
                      />
                    </div>
                    <div class="form-group">
                      <label for="ss-password">Password</label>
                      <input
                        id="ss-password"
                        type="password"
                        bind:value={$ssPassword}
                        disabled={scraping}
                      />
                    </div>
                  </div>
                </div>
              {/if}

              {#if $useTheGamesDB}
                <div class="credential-section">
                  <h4>TheGamesDB API Key</h4>
                  <p class="hint">Required for TheGamesDB</p>
                  <div class="form-group">
                    <input
                      id="tgdb-apikey"
                      type="text"
                      bind:value={$tgdbApiKey}
                      disabled={scraping}
                      placeholder="Your API key"
                    />
                  </div>
                </div>
              {/if}

              {#if !$useScreenScraper && !$useTheGamesDB}
                <p class="no-credentials">No credentials needed for selected sources.</p>
              {/if}

              <div class="option-section">
                <label class="checkbox-option">
                  <input type="checkbox" bind:checked={$skipCache} disabled={scraping} />
                  <span class="checkbox-label">
                    <strong>Skip cache</strong>
                    <span class="checkbox-hint">Clear stored data and re-scrape everything</span>
                  </span>
                </label>
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .config-section {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .settings-accordion {
    border: 1px solid var(--border-color);
    border-radius: 12px;
    overflow: hidden;
  }

  .accordion-header {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: none;
  }

  .accordion-header:hover {
    background: rgba(255, 255, 255, 0.06);
    transform: none;
    box-shadow: none;
  }

  .accordion-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-muted);
  }

  .accordion-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition: transform 0.2s ease;
  }

  .accordion-icon.expanded {
    transform: rotate(180deg);
  }

  .accordion-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    border-top: 1px solid var(--border-color);
  }

  .main-input label {
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
    display: block;
  }

  .scraper-toggles h3 {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-muted);
    margin-bottom: 0.75rem;
  }

  .toggle-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .toggle-grid .toggle-card {
    flex: 1 1 120px;
  }

  .toggle-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    padding: 0.75rem 0.5rem;
    background: rgba(255, 255, 255, 0.03);
    border: 2px solid var(--border-color);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .toggle-card:hover {
    border-color: var(--primary);
    background: rgba(108, 92, 231, 0.1);
  }

  .toggle-card.active {
    border-color: var(--primary);
    background: rgba(108, 92, 231, 0.15);
  }

  .toggle-card input[type="checkbox"] {
    display: none;
  }

  .toggle-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .toggle-card.active .toggle-label {
    color: inherit;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    border-bottom: 2px solid var(--border-color);
    padding-bottom: 0;
  }

  .tab-btn {
    flex: 1;
    padding: 0.5rem 1rem;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    color: var(--text-muted);
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: none;
  }

  .tab-btn:hover {
    color: inherit;
    transform: none;
    box-shadow: none;
  }

  .tab-btn.active {
    color: var(--primary);
    border-bottom-color: var(--primary);
  }

  .tab-content {
    min-height: 80px;
  }

  .form-group {
    margin-bottom: 0;
  }

  .form-group label {
    display: block;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-muted);
    margin-bottom: 0.4rem;
  }

  .slider-group {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .slider-group input[type="range"] {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: var(--border-color);
    appearance: none;
    -webkit-appearance: none;
  }

  .slider-group input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(108, 92, 231, 0.4);
  }

  .slider-value {
    min-width: 50px;
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--primary);
  }

  .basic-settings {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .box-art-preview {
    margin-top: 0.75rem;
    display: flex;
    flex-direction: row;
    gap: 0.5rem;
  }

  .preview-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    min-width: 0;
  }

  .preview-item img {
    width: 100%;
    max-height: 150px;
    border-radius: 6px;
    object-fit: contain;
  }

  .preview-caption {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: center;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .credential-section {
    background: rgba(255, 255, 255, 0.03);
    border-radius: 10px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .credential-section h4 {
    font-size: 0.9rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .hint {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.75rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .no-credentials {
    text-align: center;
    color: var(--text-muted);
    font-size: 0.9rem;
    padding: 1rem;
  }

  .option-section {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .checkbox-option {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    cursor: pointer;
  }

  .checkbox-option input[type="checkbox"] {
    width: 18px;
    height: 18px;
    margin-top: 2px;
    accent-color: var(--primary);
    cursor: pointer;
  }

  .checkbox-label {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .checkbox-label strong {
    font-size: 0.9rem;
  }

  .checkbox-hint {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  @media (prefers-color-scheme: light) {
    .toggle-card {
      background: rgba(0, 0, 0, 0.02);
    }

    .toggle-card.active {
      background: rgba(108, 92, 231, 0.1);
    }

    .credential-section {
      background: rgba(0, 0, 0, 0.02);
    }

    .accordion-header {
      background: rgba(0, 0, 0, 0.02);
    }

    .accordion-header:hover {
      background: rgba(0, 0, 0, 0.04);
    }
  }
</style>
