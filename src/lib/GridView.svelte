<script lang="ts">
  import type { PluginResult } from './tauri';

  let {
    results,
    selectedIndex,
    columns,
    onselect,
  }: {
    results: PluginResult[];
    selectedIndex: number;
    columns: number;
    onselect: (index: number) => void;
  } = $props();

  let pinnedCount = $derived(results.filter((r) => r.pinned).length);
</script>

<div class="grid-scroll">
  {#if pinnedCount > 0}
    <div class="section-label">Pinned</div>
    <ul class="grid" style:grid-template-columns="repeat({columns}, 1fr)">
      {#each results as result, i}
        {#if result.pinned}
          <li
            class="cell"
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
      <div class="section-label">All</div>
    {/if}
    <ul class="grid" style:grid-template-columns="repeat({columns}, 1fr)">
      {#each results as result, i}
        {#if !result.pinned}
          <li
            class="cell"
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

<style>
  .grid-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    scrollbar-width: none;
  }

  .grid-scroll::-webkit-scrollbar {
    display: none;
  }

  .section-label {
    padding: var(--space-3) var(--space-5) var(--space-1);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .grid {
    list-style: none;
    display: grid;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
  }

  .cell {
    display: flex;
    align-items: center;
    justify-content: center;
    aspect-ratio: 1;
    font-size: 2rem;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--duration-fast);
    user-select: none;
  }

  .cell:hover {
    background: var(--surface-bright);
  }

  .cell.selected {
    background: var(--bg-selected);
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
