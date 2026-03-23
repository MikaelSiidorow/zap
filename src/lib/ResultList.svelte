<script lang="ts">
  import type { PluginResult, ViewMode } from './tauri';
  import GridView from './GridView.svelte';
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
</script>

{#if view.type === 'Grid'}
  <GridView {results} {selectedIndex} columns={view.columns} {onselect} />
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
    padding: var(--space-2) 0;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
</style>
