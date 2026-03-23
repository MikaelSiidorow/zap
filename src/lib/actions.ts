import { commands, type PluginResult } from './tauri';

export type ActionOutcome =
  | { type: 'feedback'; message: string; autoHide: true }
  | { type: 'reset' }
  | { type: 'hide' }
  | { type: 'set_query'; query: string }
  | null;

export async function executeAction(result: PluginResult): Promise<ActionOutcome> {
  const action = result.action;
  if (!action) return null;

  switch (action.type) {
    case 'Copy':
      await commands.copyToClipboard(action.content);
      return { type: 'feedback', message: 'Copied to clipboard', autoHide: true };
    case 'Paste':
      await commands.pasteToFrontmost(action.content);
      return { type: 'reset' };
    case 'PasteImage':
      await commands.pasteImageToFrontmost(action.path);
      return { type: 'reset' };
    case 'SetQuery':
      return { type: 'set_query', query: action.query };
    case 'OpenUrl':
      await commands.openUrl(action.url);
      return { type: 'hide' };
    case 'Open':
    default:
      await commands.execute(result.plugin_id, result.id);
      return { type: 'hide' };
  }
}

export async function copyAction(result: PluginResult): Promise<ActionOutcome> {
  const action = result.action;
  if (!action) return null;

  switch (action.type) {
    case 'Paste':
    case 'Copy':
      await commands.copyToClipboard(action.content);
      return { type: 'feedback', message: 'Copied to clipboard', autoHide: true };
    case 'PasteImage':
      await commands.copyImageToClipboard(action.path);
      return { type: 'feedback', message: 'Image copied to clipboard', autoHide: true };
    default:
      return null;
  }
}
