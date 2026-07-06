<script lang="ts">
  import type { Lamp, Theme } from "$lib/types";
  import StatusLamp from "./StatusLamp.svelte";

  let {
    lamp,
    theme,
    ontheme,
    onfeed,
  }: { lamp: Lamp; theme: Theme; ontheme: (t: Theme) => void; onfeed: () => void } = $props();
</script>

<header class="desk-header">
  <div class="brand">
    <span class="brand-mark">CLIP2POD</span>
    <span class="label">production desk</span>
  </div>

  <StatusLamp {lamp} />

  <button class="btn-feed" onclick={onfeed}>FEED</button>

  <div class="rocker" role="group" aria-label="Theme">
    <button
      class="rocker-cell"
      class:active={theme === "light"}
      onclick={() => ontheme("light")}
      aria-pressed={theme === "light"}
    >
      DAY
    </button>
    <button
      class="rocker-cell"
      class:active={theme === "dark"}
      onclick={() => ontheme("dark")}
      aria-pressed={theme === "dark"}
    >
      NIGHT
    </button>
  </div>
</header>

<style>
  .desk-header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 16px;
    background: var(--panel);
    border-bottom: 1px solid var(--line);
  }

  .brand {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-right: auto;
  }

  .brand-mark {
    font-family: var(--mono);
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.22em;
    color: var(--text);
  }

  .rocker {
    display: flex;
    border: 1px solid var(--line);
    border-radius: 4px;
    overflow: hidden;
  }

  .rocker-cell {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.12em;
    padding: 6px 12px;
    background: var(--panel-raised);
    color: var(--muted);
    border: none;
    cursor: pointer;
  }

  .rocker-cell.active {
    background: var(--amber);
    color: var(--amber-ink);
  }

  .btn-feed {
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

  .btn-feed:hover {
    color: var(--text);
  }
</style>
