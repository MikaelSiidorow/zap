import { invoke } from '@tauri-apps/api/core';

export type Action =
  | { type: 'Open' }
  | { type: 'Copy'; content: string }
  | { type: 'OpenUrl'; url: string }
  | { type: 'SetQuery'; query: string }
  | { type: 'Paste'; content: string };

export interface PluginResult {
  id: string;
  plugin_id: string;
  title: string;
  subtitle: string | null;
  description: string | null;
  icon_path: string | null;
  score: number;
  match_indices: number[];
  action: Action;
}

export async function search(query: string): Promise<PluginResult[]> {
  return invoke('search', { query });
}

export async function execute(pluginId: string, resultId: string): Promise<void> {
  return invoke('execute', { pluginId, resultId });
}

export async function copyToClipboard(text: string): Promise<void> {
  return invoke('copy_to_clipboard', { text });
}

export async function hideWindow(): Promise<void> {
  return invoke('hide_window');
}

export async function pasteToFrontmost(text: string): Promise<void> {
  return invoke('paste_to_frontmost', { text });
}

export async function clipboardDelete(id: number): Promise<void> {
  return invoke('clipboard_delete', { id });
}

export async function clipboardTogglePin(id: number): Promise<boolean> {
  return invoke('clipboard_toggle_pin', { id });
}
