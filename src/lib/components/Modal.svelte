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
  // opener, and background inerting for free — no hand-rolled trapTab.
  $effect(() => {
    dialogEl.showModal();
  });
</script>

<dialog
  class="modal"
  bind:this={dialogEl}
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
