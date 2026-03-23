<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';

  let { value = $bindable('') }: { value: string } = $props();
  let inputEl: HTMLInputElement;
  let unlisten: (() => void) | undefined;

  onMount(async () => {
    inputEl?.focus();
    unlisten = await listen('show-window', () => {
      inputEl?.focus();
    });
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<input
  bind:this={inputEl}
  bind:value
  placeholder="Search or type ? for help"
  autofocus
  type="text"
  spellcheck="false"
  autocomplete="off"
/>

<style>
  input {
    width: 100%;
    padding: var(--space-6) var(--space-7);
    font-size: var(--text-xl);
    font-weight: 400;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-family: inherit;
  }

  input::placeholder {
    color: var(--text-muted);
  }
</style>
