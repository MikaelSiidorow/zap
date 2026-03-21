import { invoke } from '@tauri-apps/api/core';

export type Action =
  | { type: 'Open' }
  | { type: 'Copy'; content: string }
  | { type: 'OpenUrl'; url: string }
  | { type: 'SetQuery'; query: string }
  | { type: 'Paste'; content: string }
  | { type: 'PasteImage'; path: string };

export interface KeyboardHint {
  key: string;
  label: string;
}

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

export async function pluginHints(pluginId: string): Promise<KeyboardHint[]> {
  return invoke('plugin_hints', { pluginId });
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

export async function pasteImageToFrontmost(path: string): Promise<void> {
  return invoke('paste_image_to_frontmost', { path });
}

export async function copyImageToClipboard(path: string): Promise<void> {
  return invoke('copy_image_to_clipboard', { path });
}

export async function clipboardDelete(id: number): Promise<void> {
  return invoke('clipboard_delete', { id });
}

export async function clipboardTogglePin(id: number): Promise<boolean> {
  return invoke('clipboard_toggle_pin', { id });
}
