const GRID_COLUMNS = 8;

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
  pluginId: string | null,
  view: 'list' | 'grid' = 'list',
): KeyAction {
  // Clipboard-specific shortcuts
  if (pluginId === 'clipboard' && resultCount > 0) {
    // Delete: Delete key on Linux/Windows, Cmd+Backspace on Mac
    if (key === 'Delete' || (key === 'Backspace' && metaKey)) {
      return { type: 'delete', index: selectedIndex };
    }
    if (key === 'p' && (ctrlKey || metaKey)) {
      return { type: 'toggle_pin', index: selectedIndex };
    }
    if (key === 'Enter' && shiftKey) {
      return { type: 'copy', index: selectedIndex };
    }
  }

  // Emoji pin shortcut
  if (pluginId === 'emoji' && resultCount > 0) {
    if (key === 'p' && (ctrlKey || metaKey)) {
      return { type: 'toggle_pin', index: selectedIndex };
    }
  }

  // Grid navigation
  if (view === 'grid' && resultCount > 0) {
    switch (key) {
      case 'ArrowRight':
        return {
          type: 'move',
          index: Math.min(selectedIndex + 1, resultCount - 1),
        };
      case 'ArrowLeft':
        return {
          type: 'move',
          index: Math.max(selectedIndex - 1, 0),
        };
      case 'ArrowDown':
        return {
          type: 'move',
          index: Math.min(selectedIndex + GRID_COLUMNS, resultCount - 1),
        };
      case 'ArrowUp':
        return {
          type: 'move',
          index: Math.max(selectedIndex - GRID_COLUMNS, 0),
        };
    }
  }

  switch (key) {
    case 'ArrowDown':
      return {
        type: 'move',
        index: (selectedIndex + 1) % Math.max(resultCount, 1),
      };
    case 'ArrowUp':
      return {
        type: 'move',
        index:
          (selectedIndex - 1 + Math.max(resultCount, 1)) %
          Math.max(resultCount, 1),
      };
    case 'Enter':
      return resultCount > 0
        ? { type: 'select', index: selectedIndex }
        : null;
    case 'Escape':
      return { type: 'hide' };
    default:
      return null;
  }
}
