<script lang="ts">
  import type { Lamp } from "$lib/types";

  let { lamp }: { lamp: Lamp } = $props();

  const text = $derived(
    lamp.state === "OnAir" ? "ON AIR" : lamp.state === "Queued" ? `QUEUED ${lamp.queued}` : "IDLE",
  );
</script>

<div class="lamp" data-state={lamp.state} role="status" aria-live="polite">
  <span class="bulb"></span>
  <span class="label lamp-text">{text}</span>
</div>

<style>
  .lamp {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--line);
    border-radius: 4px;
    background: var(--panel-raised);
    padding: 6px 12px;
    min-width: 108px;
  }

  .bulb {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--lamp-off);
    transition: background 200ms, box-shadow 200ms;
  }

  .lamp[data-state="Queued"] .bulb {
    background: var(--amber);
    box-shadow: 0 0 8px var(--amber);
  }

  .lamp[data-state="OnAir"] .bulb {
    background: var(--onair);
    box-shadow: 0 0 10px var(--onair);
    animation: onair-pulse 1.4s ease-in-out infinite;
  }

  .lamp[data-state="OnAir"] .lamp-text {
    color: var(--onair);
  }

  .lamp[data-state="Queued"] .lamp-text {
    color: var(--amber);
  }

  @keyframes onair-pulse {
    50% {
      box-shadow: 0 0 3px var(--onair);
      opacity: 0.75;
    }
  }
</style>
