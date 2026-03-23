// Re-export generated types and commands from tauri-specta bindings.
// All types (Action, PluginResult, SearchResponse, etc.) and command
// wrappers are auto-generated — do not define them manually here.

export { commands } from './bindings';
export type {
  Action,
  Capability,
  KeyboardHint,
  PluginResult,
  SearchResponse,
  ViewMode,
} from './bindings';
