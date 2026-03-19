import { invoke } from '@tauri-apps/api/core';

export interface PluginResult {
  id: string;
  plugin_id: string;
  title: string;
  subtitle: string | null;
  icon_path: string | null;
  score: number;
  match_indices: number[];
}

export async function search(query: string): Promise<PluginResult[]> {
  return invoke('search', { query });
}

export async function execute(pluginId: string, resultId: string): Promise<void> {
  return invoke('execute', { pluginId, resultId });
}

export async function hideWindow(): Promise<void> {
  return invoke('hide_window');
}
