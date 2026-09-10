<script lang="ts">
  import type { Lamp } from "$lib/types";

  let { lamp, label }: { lamp: Lamp; label?: string } = $props();

  const text = $derived(
    label ??
      (lamp.state === "OnAir"
        ? "RENDERING"
        : lamp.state === "Queued"
          ? `QUEUED ${lamp.queued}`
          : "IDLE"),
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
    background: var(--idle);
    transition: background 200ms, box-shadow 200ms;
  }

  .lamp[data-state="Queued"] .bulb {
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }

  .lamp[data-state="OnAir"] .bulb {
    background: var(--danger);
    box-shadow: 0 0 10px var(--danger);
    animation: onair-pulse 1.4s ease-in-out infinite;
  }

  .lamp[data-state="OnAir"] .lamp-text {
    color: var(--danger);
  }

  .lamp[data-state="Queued"] .lamp-text {
    color: var(--accent);
  }

  @keyframes onair-pulse {
    50% {
      box-shadow: 0 0 3px var(--danger);
      opacity: 0.75;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    /* Drop the pulse; a steady bright halo still separates ON AIR from QUEUED. */
    .lamp[data-state="OnAir"] .bulb {
      animation: none;
      box-shadow: 0 0 12px var(--danger);
    }
  }
</style>
