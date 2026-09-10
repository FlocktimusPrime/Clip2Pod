<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    onclose,
    children,
    footer,
  }: { title: string; onclose: () => void; children: Snippet; footer?: Snippet } = $props();

  let dialogEl: HTMLDialogElement;
  const titleId = `modal-title-${Math.random().toString(36).slice(2)}`;

  // showModal() gives us the focus trap, Esc-to-close, focus return to the
  // opener, and background inerting for free — no hand-rolled trapTab. It
  // also focuses the first control (Close); move focus to the panel instead
  // so opening doesn't flash a ring on a button, while the dialog's
  // aria-labelledby still gets announced.
  $effect(() => {
    dialogEl.showModal();
    dialogEl.focus();
  });
</script>

<dialog
  class="modal"
  bind:this={dialogEl}
  tabindex="-1"
  aria-labelledby={titleId}
  onclose={onclose}
  onclick={(e) => {
    if (e.target === dialogEl) onclose();
  }}
>
  <header>
    <h2 class="label" id={titleId}>{title}</h2>
    <button class="btn" onclick={onclose}>Close <kbd>Esc</kbd></button>
  </header>
  <div class="body">
    {@render children()}
  </div>
  {#if footer}
    <footer>{@render footer()}</footer>
  {/if}
</dialog>
