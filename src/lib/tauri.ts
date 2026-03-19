import { invoke } from '@tauri-apps/api/core';

export interface SearchResult {
  id: string;
  name: string;
  exec_path: string;
  icon_path: string | null;
  category: string | null;
  score: number;
  match_indices: number[];
}

export async function search(query: string): Promise<SearchResult[]> {
  return invoke('search', { query });
}

export async function launch(id: string): Promise<void> {
  return invoke('launch', { id });
}

export async function hideWindow(): Promise<void> {
  return invoke('hide_window');
}