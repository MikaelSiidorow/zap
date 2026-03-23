<script lang="ts">
  import type { PluginResult, ViewMode } from './tauri';
  import ResultItem from './ResultItem.svelte';

  let {
    results,
    selectedIndex,
    onselect,
    view = { type: 'List' } as ViewMode,
  }: {
    results: PluginResult[];
    selectedIndex: number;
    onselect: (index: number) => void;
    view?: ViewMode;
  } = $props();

  let pinnedCount = $derived(results.filter((r) => r.pinned).length);
  let gridColumns = $derived(view.type === 'Grid' ? view.columns : 8);
</script>

{#if view.type === 'Grid'}
  <div class="grid-scroll">
    {#if pinnedCount > 0}
      <div class="grid-section-label">Pinned</div>
      <ul class="grid" style:grid-template-columns="repeat({gridColumns}, 1fr)">
        {#each results as result, i}
          {#if result.pinned}
            <li
              class="grid-item"
              class:selected={i === selectedIndex}
              onclick={() => onselect(i)}
              title={result.title}
              role="option"
              aria-selected={i === selectedIndex}
              tabindex="-1"
            >
              {#if result.action?.type === 'Copy'}
                {result.action.content}
              {:else}
                {result.title[0]}
              {/if}
            </li>
          {/if}
        {/each}
      </ul>
    {/if}
    {#if pinnedCount < results.length}
      {#if pinnedCount > 0}
        <div class="grid-section-label">All</div>
      {/if}
      <ul class="grid" style:grid-template-columns="repeat({gridColumns}, 1fr)">
        {#each results as result, i}
          {#if !result.pinned}
            <li
              class="grid-item"
              class:selected={i === selectedIndex}
              onclick={() => onselect(i)}
              title={result.title}
              role="option"
              aria-selected={i === selectedIndex}
              tabindex="-1"
            >
              {#if result.action?.type === 'Copy'}
                {result.action.content}
              {:else}
                {result.title[0]}
              {/if}
            </li>
          {/if}
        {/each}
      </ul>
    {/if}
  </div>
{:else}
  <ul class="result-list">
    {#each results as result, i}
      <ResultItem
        {result}
        selected={i === selectedIndex}
        onclick={() => onselect(i)}
      />
    {/each}
  </ul>
{/if}

<style>
  .result-list {
    list-style: none;
    padding: 4px 0;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .grid-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    scrollbar-width: none;
  }

  .grid-scroll::-webkit-scrollbar {
    display: none;
  }

  .grid-section-label {
    padding: 6px 12px 2px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .grid {
    list-style: none;
    display: grid;
    gap: 4px;
    padding: 4px 8px;
  }

  .grid-item {
    display: flex;
    align-items: center;
    justify-content: center;
    aspect-ratio: 1;
    font-size: 32px;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.1s;
    user-select: none;
  }

  .grid-item:hover {
    background: var(--surface-bright);
  }

  .grid-item.selected {
    background: var(--bg-selected);
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
