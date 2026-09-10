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

  const tabs: { id: TabName; text: string }[] = [
    { id: "narrate", text: "NARRATE" },
    { id: "rip", text: "RIP" },
  ];
</script>

<header class="app-header">
  <h1 class="brand-mark">CLIP2POD</h1>

  <div class="tabs" role="tablist" aria-label="Mode">
    {#each tabs as t (t.id)}
      <button
        class="tab"
        role="tab"
        aria-selected={tab === t.id}
        onclick={() => ontab(t.id)}
      >
        {t.text}
      </button>
    {/each}
  </div>

  <StatusLamp {lamp} {label} />

  <div class="actions">
    <button class="hbtn" onclick={onfeed}>FEED</button>
    <button class="hbtn" onclick={onlog}>LOG</button>
    <button class="hbtn" onclick={onsettings}>SETTINGS</button>
  </div>
</header>

<style>
  .app-header {
    display: flex;
    align-items: center;
    gap: 16px;
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

  .tabs {
    display: flex;
    gap: 2px;
    margin-right: auto;
    padding: 2px;
    background: var(--panel-raised);
    border: 1px solid var(--line);
    border-radius: 4px;
  }

  .tab {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.12em;
    padding: 5px 12px;
    background: transparent;
    color: var(--muted);
    border: none;
    border-radius: 3px;
    cursor: pointer;
  }

  .tab:hover {
    color: var(--text);
  }

  .tab[aria-selected="true"] {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .hbtn {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.12em;
    padding: 6px 12px;
    background: var(--panel-raised);
    color: var(--muted);
    border: 1px solid var(--line);
    border-radius: 4px;
    cursor: pointer;
  }

  .hbtn:hover {
    color: var(--text);
  }
</style>
