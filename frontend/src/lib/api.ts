import type { AppState, GameData, ListDirectoriesResponse, ProgressUpdate, ScrapeConfig, SaveSettingsConfig, SaveSettingsResponse } from './types';

export async function loadState(romsPath: string): Promise<AppState> {
  const response = await fetch('/api/state', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ romsPath }),
  });
  return await response.json();
}

export async function loadGames(): Promise<string[]> {
  const response = await fetch('/api/games');
  return await response.json();
}

export async function loadGameByRomName(romName: string): Promise<GameData> {
  const response = await fetch(`/api/games/${romName}`);
  if (!response.ok) {
    throw new Error(`Failed to load game: ${response.statusText}`);
  }
  return await response.json();
}

export async function saveSettings(config: SaveSettingsConfig): Promise<SaveSettingsResponse> {
  const response = await fetch('/api/settings', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(config),
  });
  return await response.json();
}

export async function startScraping(config: ScrapeConfig): Promise<{ success: boolean; message: string }> {
  const response = await fetch('/api/scrape', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(config),
  });
  return await response.json();
}

export async function stopScraping(): Promise<{ success: boolean; message: string }> {
  const response = await fetch('/api/stop', {
    method: 'POST',
  });
  return await response.json();
}

export async function listDirectories(path: string): Promise<ListDirectoriesResponse> {
  const response = await fetch('/api/directories', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ path }),
  });
  if (!response.ok) {
    throw new Error(`Failed to list directories: ${response.statusText}`);
  }
  return await response.json();
}

export function connectToProgressStream(
  onMessage: (update: ProgressUpdate) => void,
  onError: () => void
): EventSource {
  const eventSource = new EventSource('/api/progress');

  eventSource.onmessage = (event: MessageEvent) => {
    if (event.data === 'keep-alive') return;

    try {
      const update: ProgressUpdate = JSON.parse(event.data);
      onMessage(update);
    } catch (e) {
      console.warn('Received non-JSON message:', event.data);
    }
  };

  eventSource.onerror = () => {
    console.log('EventSource connection closed');
    eventSource.close();
    onError();
  };

  return eventSource;
}
