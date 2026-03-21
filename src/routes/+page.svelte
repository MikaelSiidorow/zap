<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import SearchBar from '$lib/SearchBar.svelte';
  import ResultList from '$lib/ResultList.svelte';
  import { search, execute, copyToClipboard, hideWindow, pasteToFrontmost, pasteImageToFrontmost, copyImageToClipboard, clipboardDelete, clipboardTogglePin, pluginHints, type PluginResult, type KeyboardHint } from '$lib/tauri';
  import { handleKey } from '$lib/keys';

  let query = $state('');
  let results = $state<PluginResult[]>([]);
  let selectedIndex = $state(0);
  let feedback = $state<string | null>(null);
  let hints = $state<KeyboardHint[]>([]);
  let unlisten: (() => void) | undefined;

  $effect(() => {
    const q = query;
    search(q).then((r) => {
      if (query === q) {
        results = r;
        selectedIndex = 0;
      }
    });
  });

  // Fetch hints when active plugin changes
  let activePluginId = $derived(results[0]?.plugin_id ?? null);
  $effect(() => {
    const pid = activePluginId;
    if (pid) {
      pluginHints(pid).then((h) => { hints = h; });
    } else {
      hints = [];
    }
  });

  onMount(async () => {
    unlisten = await listen('show-window', () => {
      query = '';
      results = [];
      selectedIndex = 0;
      feedback = null;
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  function reset() {
    query = '';
    results = [];
    selectedIndex = 0;
    feedback = null;
  }

  function hide() {
    reset();
    hideWindow();
  }

  async function activateResult(result: PluginResult) {
    switch (result.action.type) {
      case 'Copy':
        await copyToClipboard(result.action.content);
        feedback = 'Copied to clipboard';
        setTimeout(hide, 600);
        break;
      case 'Paste':
        await pasteToFrontmost(result.action.content);
        reset();
        break;
      case 'PasteImage':
        await pasteImageToFrontmost(result.action.path);
        reset();
        break;
      case 'SetQuery':
        query = result.action.query;
        break;
      case 'OpenUrl':
        // Future: open URL in browser
        hide();
        break;
      case 'Open':
      default:
        await execute(result.plugin_id, result.id);
        hide();
        break;
    }
  }

  async function refreshSearch() {
    const q = query;
    const r = await search(q);
    if (query === q) {
      results = r;
      if (selectedIndex >= results.length) {
        selectedIndex = Math.max(0, results.length - 1);
      }
    }
  }

  async function copyResult(result: PluginResult) {
    switch (result.action.type) {
      case 'Paste':
      case 'Copy':
        await copyToClipboard(result.action.content);
        feedback = 'Copied to clipboard';
        setTimeout(hide, 600);
        break;
      case 'PasteImage':
        await copyImageToClipboard(result.action.path);
        feedback = 'Image copied to clipboard';
        setTimeout(hide, 600);
        break;
      default:
        break;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    const selectedResult = results[selectedIndex] ?? null;
    const pluginId = selectedResult?.plugin_id ?? null;
    const action = handleKey(event.key, selectedIndex, results.length, event.ctrlKey, event.metaKey, event.shiftKey, pluginId);
    if (!action) return;

    event.preventDefault();
    switch (action.type) {
      case 'move':
        selectedIndex = action.index;
        break;
      case 'select':
        if (results[action.index]) {
          activateResult(results[action.index]);
        }
        break;
      case 'copy':
        if (results[action.index]) {
          copyResult(results[action.index]);
        }
        break;
      case 'hide':
        hide();
        break;
      case 'delete':
        if (selectedResult) {
          clipboardDelete(Number(selectedResult.id)).then(refreshSearch);
        }
        break;
      case 'toggle_pin':
        if (selectedResult) {
          clipboardTogglePin(Number(selectedResult.id)).then(refreshSearch);
        }
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={hide}>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <main onclick={(e) => e.stopPropagation()}>
    <SearchBar bind:value={query} />
    {#if feedback}
      <div class="divider"></div>
      <div class="feedback">{feedback}</div>
    {:else if results.length > 0}
      <div class="divider"></div>
      <ResultList {results} {selectedIndex} onselect={(i) => activateResult(results[i])} />
      {#if hints.length > 0}
        <div class="hints">
          {#each hints as hint}
            <span><kbd>{hint.key}</kbd> {hint.label}</span>
          {/each}
        </div>
      {/if}
    {/if}
  </main>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    justify-content: center;
    padding-top: 8px;
  }

  main {
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: 12px;
    border: 1px solid var(--border);
    overflow: hidden;
    width: 680px;
    align-self: flex-start;
  }

  .divider {
    height: 1px;
    background: var(--border);
  }

  .feedback {
    padding: 16px 20px;
    font-size: 14px;
    color: var(--text-muted);
    text-align: center;
  }

  .hints {
    display: flex;
    gap: 16px;
    padding: 6px 16px;
    border-top: 1px solid var(--border);
    font-size: 11px;
    color: var(--text-muted);
  }

  .hints kbd {
    font-family: inherit;
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    border: 1px solid var(--border);
    background: var(--bg-secondary, rgba(255, 255, 255, 0.06));
    margin-right: 4px;
  }
</style>
