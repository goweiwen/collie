import { writable, type Writable } from 'svelte/store';

// Reusable store for localStorage-synced state
function localStorageStore<T>(key: string, defaultValue: T): Writable<T> {
  const stored = localStorage.getItem(key);
  let initial: T;

  if (stored !== null) {
    try {
      initial = JSON.parse(stored) as T;
    } catch {
      initial = stored as T;
    }
  } else {
    initial = defaultValue;
  }

  const store = writable<T>(initial);

  store.subscribe(value => {
    localStorage.setItem(key, JSON.stringify(value));
  });

  return store;
}

// Application settings stores
export const romsPath = localStorageStore('romsPath', '/mnt/SDCARD/Roms');
export const boxArtWidth = localStorageStore('boxArtWidth', 250);

// Metadata backend selection
export const useScreenScraper = localStorageStore('useScreenScraper', true);
export const useTheGamesDB = localStorageStore('useTheGamesDB', false);

// ScreenScraper config
export const ssUsername = localStorageStore('ssUsername', '');
export const ssPassword = localStorageStore('ssPassword', '');
export const boxArtType = localStorageStore('boxArtType', 'box-2D');

// Box art type options for ScreenScraper
export const boxArtTypes = [
  { value: 'box-2D', label: 'Box: Front' },
  { value: 'box-3D', label: '3D Box' },
  { value: 'mixrbv1', label: 'Mix RBv1' },
  { value: 'mixrbv2', label: 'Mix RBv2' },
  { value: 'ss', label: 'Screenshot' },
  { value: 'sstitle', label: 'Screenshot Title' },
  { value: 'support-2D', label: 'Support: 2D' },
  { value: '', label: '───────────', disabled: true },
  { value: 'box-2D-back', label: 'Box: Back' },
  { value: 'box-2D-side', label: 'Box: Side' },
  { value: 'box-texture', label: 'Box: Texture' },
  { value: 'wheel', label: 'Wheel' },
  { value: 'fanart', label: 'Fan Art' },
  { value: 'steamgrid', label: 'Steam Grid' },
  { value: 'screenmarquee', label: 'Screen Marquee' },
  { value: 'support-texture', label: 'Support: Texture' },
];

// Preview games for box art type selector
export const previewGames = [
  { name: 'Apotris', platformId: 12, gameId: 490139, region: 'wor' },
  { name: 'Final Fantasy VII', platformId: 57, gameId: 19249, region: 'us' },
  { name: 'Chrono Trigger', platformId: 4, gameId: 2143, region: 'us' },
];

export const boxArtPreviewUrl = (mediaType: string, platformId: number, gameId: number, region: string, hd: number = 1, num: string = '') =>
  `https://www.screenscraper.fr/image.php?plateformid=${platformId}&gameid=${gameId}&region=${region}&hd=${hd}&num=${num}&maxwidth=150&maxheight=150&media=${mediaType}`;

// TheGamesDB config
export const tgdbApiKey = localStorageStore('tgdbApiKey', '');

// Guide backend selection
export const useGameFAQs = localStorageStore('useGameFAQs', true);

// Advanced settings (not persisted)
export const skipCache = writable(false);
