export type KeyAction =
  | { type: 'move'; index: number }
  | { type: 'select'; index: number }
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
  pluginId: string | null,
): KeyAction {
  // Clipboard-specific shortcuts
  if (pluginId === 'clipboard' && resultCount > 0) {
    if (key === 'Delete' || key === 'Backspace') {
      return { type: 'delete', index: selectedIndex };
    }
    if (key === 'p' && (ctrlKey || metaKey)) {
      return { type: 'toggle_pin', index: selectedIndex };
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
