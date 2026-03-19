<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import SearchBar from '$lib/SearchBar.svelte';
  import ResultList from '$lib/ResultList.svelte';
  import { search, launch, hideWindow, type SearchResult } from '$lib/tauri';
  import { handleKey } from '$lib/keys';

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);
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

  onMount(async () => {
    unlisten = await listen('show-window', () => {
      query = '';
      results = [];
      selectedIndex = 0;
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  function hide() {
    query = '';
    results = [];
    selectedIndex = 0;
    hideWindow();
  }

  function handleKeydown(event: KeyboardEvent) {
    const action = handleKey(event.key, selectedIndex, results.length);
    if (!action) return;

    event.preventDefault();
    switch (action.type) {
      case 'move':
        selectedIndex = action.index;
        break;
      case 'select':
        if (results[action.index]) {
          launch(results[action.index].id);
          hide();
        }
        break;
      case 'hide':
        hide();
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
    {#if results.length > 0}
      <div class="divider"></div>
      <ResultList {results} {selectedIndex} onselect={(i) => {
        launch(results[i].id);
        hide();
      }} />
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
</style>
