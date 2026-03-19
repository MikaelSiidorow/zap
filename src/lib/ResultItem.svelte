<script lang="ts">
  import type { PluginResult } from './tauri';

  let {
    result,
    selected,
    onclick,
  }: {
    result: PluginResult;
    selected: boolean;
    onclick: () => void;
  } = $props();

  let iconError = $state(false);

  function iconUrl(path: string): string {
    return `icon://localhost/${encodeURIComponent(path)}`;
  }

  function highlightedChars(): { char: string; highlight: boolean }[] {
    const indices = new Set(result.match_indices);
    return [...result.title].map((char, i) => ({
      char,
      highlight: indices.has(i),
    }));
  }
</script>

<li
  class="result-item"
  class:selected
  onclick={onclick}
  onkeydown={(e) => e.key === 'Enter' && onclick()}
  role="option"
  aria-selected={selected}
  tabindex="-1"
>
  {#if result.icon_path && !iconError}
    <img src={iconUrl(result.icon_path)} alt="" class="icon" onerror={() => iconError = true} />
  {:else}
    <div class="icon-placeholder">{result.title[0]}</div>
  {/if}
  <div class="info">
    <span class="name">
      {#each highlightedChars() as { char, highlight }}
        {#if highlight}
          <mark>{char}</mark>
        {:else}
          {char}
        {/if}
      {/each}
    </span>
    {#if result.subtitle}
      <span class="category">{result.subtitle}</span>
    {/if}
  </div>
</li>

<style>
  .result-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 20px;
    cursor: pointer;
  }

  .result-item:hover,
  .result-item.selected {
    background: var(--bg-selected);
  }

  .icon {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    object-fit: contain;
    flex-shrink: 0;
  }

  .icon-placeholder {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background: var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .info {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }

  .name {
    font-size: 15px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .category {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }
</style>
