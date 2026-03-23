import type { Capability, ViewMode } from './tauri';

export type KeyAction =
  | { type: 'move'; index: number }
  | { type: 'select'; index: number }
  | { type: 'copy'; index: number }
  | { type: 'hide' }
  | { type: 'delete'; index: number }
  | { type: 'toggle_pin'; index: number }
  | null;

export function handleKey(
  key: string,
  selectedIndex: number,
  resultCount: number,
  ctrlKey: boolean,
  metaKey: boolean,
  shiftKey: boolean,
  capabilities: Capability[],
  view: ViewMode,
): KeyAction {
  const caps = new Set(capabilities);

  if (resultCount > 0) {
    if (caps.has('Delete') && (key === 'Delete' || (key === 'Backspace' && metaKey))) {
      return { type: 'delete', index: selectedIndex };
    }
    if (caps.has('Pin') && key === 'p' && (ctrlKey || metaKey)) {
      return { type: 'toggle_pin', index: selectedIndex };
    }
    if (caps.has('Copy') && key === 'Enter' && shiftKey) {
      return { type: 'copy', index: selectedIndex };
    }
  }

  // Grid navigation
  if (view.type === 'Grid' && resultCount > 0) {
    const cols = view.columns;
    switch (key) {
      case 'ArrowRight':
        return { type: 'move', index: Math.min(selectedIndex + 1, resultCount - 1) };
      case 'ArrowLeft':
        return { type: 'move', index: Math.max(selectedIndex - 1, 0) };
      case 'ArrowDown':
        return { type: 'move', index: Math.min(selectedIndex + cols, resultCount - 1) };
      case 'ArrowUp':
        return { type: 'move', index: Math.max(selectedIndex - cols, 0) };
    }
  }

  switch (key) {
    case 'ArrowDown':
      return { type: 'move', index: (selectedIndex + 1) % Math.max(resultCount, 1) };
    case 'ArrowUp':
      return {
        type: 'move',
        index: (selectedIndex - 1 + Math.max(resultCount, 1)) % Math.max(resultCount, 1),
      };
    case 'Enter':
      return resultCount > 0 ? { type: 'select', index: selectedIndex } : null;
    case 'Escape':
      return { type: 'hide' };
    default:
      return null;
  }
}
