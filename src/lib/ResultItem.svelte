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
  {:else if result.icon_path}
    <div class="icon-placeholder">{result.title[0]}</div>
  {/if}
  <div class="info" class:stacked={result.description}>
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
      <span class="subtitle">{result.subtitle}</span>
    {/if}
    {#if result.description}
      <span class="description">{result.description}</span>
    {/if}
  </div>
</li>

<style>
  .result-item {
    display: flex;
    align-items: center;
    gap: var(--space-5);
    padding: var(--space-4) var(--space-7);
    cursor: pointer;
  }

  .result-item:hover,
  .result-item.selected {
    background: var(--bg-selected);
  }

  .icon {
    width: var(--icon-size);
    height: var(--icon-size);
    border-radius: var(--radius-md);
    object-fit: contain;
    flex-shrink: 0;
  }

  .icon-placeholder {
    width: var(--icon-size);
    height: var(--icon-size);
    border-radius: var(--radius-md);
    background: var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .info {
    display: flex;
    align-items: baseline;
    gap: var(--space-4);
    min-width: 0;
  }

  .info.stacked {
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-1);
  }

  .name {
    font-size: var(--text-lg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .subtitle {
    font-size: var(--text-sm);
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .info.stacked .subtitle {
    font-size: var(--text-base);
    font-family: var(--font-mono);
  }

  .description {
    font-size: var(--text-sm);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
