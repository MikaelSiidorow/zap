<script lang="ts">
  import type { PluginResult } from './tauri';
  import ResultItem from './ResultItem.svelte';

  let {
    results,
    selectedIndex,
    onselect,
  }: {
    results: PluginResult[];
    selectedIndex: number;
    onselect: (index: number) => void;
  } = $props();
</script>

<ul class="result-list">
  {#each results as result, i}
    <ResultItem
      {result}
      selected={i === selectedIndex}
      onclick={() => onselect(i)}
    />
  {/each}
</ul>

<style>
  .result-list {
    list-style: none;
    padding: 4px 0;
    max-height: 360px;
    overflow-y: auto;
  }
</style>
