<script lang="ts">
  import type { Lamp } from "$lib/types";
  import StatusLamp from "./StatusLamp.svelte";

  let { lamp, onfeed }: { lamp: Lamp; onfeed: () => void } = $props();
</script>

<header class="app-header">
  <div class="brand">
    <span class="brand-mark">CLIP2POD</span>
    <span class="tagline"
      >Turn any article into a narrated MP3 — and listen to it as your own
      private podcast feed.</span
    >
  </div>

  <StatusLamp {lamp} />

  <button class="btn-feed" onclick={onfeed}>FEED</button>
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

  .brand {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-right: auto;
    min-width: 0;
  }

  .brand-mark {
    font-family: var(--mono);
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.22em;
    color: var(--text);
    flex-shrink: 0;
  }

  .tagline {
    font-size: 11.5px;
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (max-width: 900px) {
    .tagline {
      display: none;
    }
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
