<script lang="ts">
  import type { Lamp } from "$lib/types";
  import type { TabName } from "$lib/stores.svelte";
  import StatusLamp from "./StatusLamp.svelte";

  let {
    tab,
    ontab,
    lamp,
    label,
    onfeed,
    onlog,
    onsettings,
  }: {
    tab: TabName;
    ontab: (t: TabName) => void;
    lamp: Lamp;
    label: string;
    onfeed: () => void;
    onlog: () => void;
    onsettings: () => void;
  } = $props();

  const modes: { id: TabName; text: string }[] = [
    { id: "narrate", text: "NARRATE" },
    { id: "rip", text: "RIP" },
  ];
</script>

<header class="app-header">
  <h1 class="brand-mark">CLIP2POD</h1>

  <div class="seg" role="group" aria-label="Mode">
    {#each modes as m (m.id)}
      <button
        class="seg-cell"
        aria-pressed={tab === m.id}
        onclick={() => ontab(m.id)}
      >
        {m.text}
      </button>
    {/each}
  </div>

  <StatusLamp {lamp} {label} />

  <div class="actions">
    <button class="btn" onclick={onfeed}>FEED</button>
    <button class="btn" onclick={onlog}>LOG</button>
    <button class="btn" onclick={onsettings}>SETTINGS</button>
  </div>
</header>

<style>
  .app-header {
    display: flex;
    align-items: center;
    gap: 16px;
    row-gap: 8px;
    flex-wrap: wrap;
    padding: 10px 16px;
    background: var(--panel);
    border-bottom: 1px solid var(--line);
  }

  .brand-mark {
    margin: 0;
    font-family: var(--mono);
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.22em;
    color: var(--text);
    flex-shrink: 0;
  }

  /* Segmented control per DESIGN.md: one bordered box, borderless cells
     divided only by fill, active cell flips to the accent. */
  .seg {
    display: flex;
    margin-right: auto;
    border: 1px solid var(--line);
    border-radius: 4px;
    overflow: hidden;
    height: 31px;
  }

  .seg-cell {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.14em;
    padding: 0 14px;
    background: var(--panel-raised);
    color: var(--muted);
    border: none;
    cursor: pointer;
    transition: color 120ms;
  }

  .seg-cell:hover {
    color: var(--text);
  }

  .seg-cell[aria-pressed="true"] {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .seg-cell:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .actions .btn {
    padding: 6px 12px;
  }
</style>
