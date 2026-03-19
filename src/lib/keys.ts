export type KeyAction =
  | { type: 'move'; index: number }
  | { type: 'select'; index: number }
  | { type: 'hide' }
  | null;

export function handleKey(
  key: string,
  selectedIndex: number,
  resultCount: number,
): KeyAction {
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
