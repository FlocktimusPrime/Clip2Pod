<script lang="ts">
  import type { Lamp } from "$lib/types";
  import StatusLamp from "./StatusLamp.svelte";

  let { lamp, onfeed }: { lamp: Lamp; onfeed: () => void } = $props();
</script>

<header class="desk-header">
  <div class="brand">
    <span class="brand-mark">CLIP2POD</span>
    <span class="label">production desk</span>
  </div>

  <StatusLamp {lamp} />

  <button class="btn-feed" onclick={onfeed}>FEED</button>
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
