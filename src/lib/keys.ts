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
