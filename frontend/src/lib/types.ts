export type ScrapeStatus = 'pending' | 'searching' | 'success' | 'failed' | 'skipped';

export interface GameMetadata {
  status: ScrapeStatus;
  name?: string;
  developer?: string;
  publisher?: string;
  genre?: string;
  release_date?: string;
  rating?: string;
  image_path?: string;
  error_message?: string;
}

export interface GameGuides {
  status: ScrapeStatus;
  count?: number;
}

export interface GameData {
  rom_name: string;
  status?: ScrapeStatus;
  metadata: GameMetadata;
  guides: GameGuides;
}

export interface ScrapeConfig {
  romsPath: string;
  boxArtWidth: number;
  skipCache: boolean;
  metadataBackends: {
    screenscraper: {
      username: string | null;
      password: string | null;
    } | null;
    thegamesdb: {
      apiKey: string;
    } | null;
  };
  guideBackends: {
    gamefaqs: boolean | null;
  };
}

export interface ProgressUpdate {
  completed: number;
  total: number;
  success_count: number;
  fail_count: number;
  skip_count: number;
  message: string;
  game_update?: GameData;
}

export interface AppState {
  scraping: boolean;
  progress: number;
  total_games: number;
  success_count: number;
  fail_count: number;
  skip_count: number;
  current_message: string;
}

export interface SaveSettingsConfig {
  romsPath: string;
}

export interface SaveSettingsResponse {
  success: boolean;
  message: string;
  state: AppState;
}
