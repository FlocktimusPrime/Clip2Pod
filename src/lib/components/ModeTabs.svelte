<script lang="ts">
  import type { TabName } from "$lib/stores.svelte";

  // Narrate | Rip switcher inside a dialog (Feed, Log). It only picks which
  // panel the dialog shows — it never changes the app's mode tab. The parent
  // renders the panels with ids `${idPrefix}-panel-${mode}`.
  let {
    value = $bindable(),
    label,
    idPrefix,
  }: { value: TabName; label: string; idPrefix: string } = $props();

  const modes: { id: TabName; text: string }[] = [
    { id: "narrate", text: "NARRATE" },
    { id: "rip", text: "RIP" },
  ];

  let buttons: HTMLButtonElement[] = [];

  // WAI-ARIA tabs, automatic activation: arrows / Home / End move focus and
  // select in one step. Only the selected tab is in the Tab order.
  function onkeydown(e: KeyboardEvent, i: number) {
    let next: number;
    if (e.key === "ArrowRight") next = (i + 1) % modes.length;
    else if (e.key === "ArrowLeft") next = (i - 1 + modes.length) % modes.length;
    else if (e.key === "Home") next = 0;
    else if (e.key === "End") next = modes.length - 1;
    else return;
    e.preventDefault();
    value = modes[next].id;
    buttons[next]?.focus();
  }
</script>

<div class="seg" role="tablist" aria-label={label}>
  {#each modes as m, i (m.id)}
    <button
      class="seg-cell"
      role="tab"
      id="{idPrefix}-tab-{m.id}"
      aria-selected={value === m.id}
      aria-controls="{idPrefix}-panel-{m.id}"
      tabindex={value === m.id ? 0 : -1}
      bind:this={buttons[i]}
      onclick={() => (value = m.id)}
      onkeydown={(e) => onkeydown(e, i)}
    >
      {m.text}
    </button>
  {/each}
</div>

<style>
  /* The header's mode control (DESIGN.md segmented control), a size down. */
  .seg {
    display: inline-flex;
    border: 1px solid var(--line);
    border-radius: 4px;
    overflow: hidden;
    height: 27px;
    margin-bottom: 12px;
  }

  .seg-cell {
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.14em;
    padding: 0 12px;
    background: var(--panel-raised);
    color: var(--muted);
    border: none;
    cursor: pointer;
    transition: color 120ms;
  }

  .seg-cell:hover {
    color: var(--text);
  }

  .seg-cell[aria-selected="true"] {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .seg-cell:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
