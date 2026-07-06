<script lang="ts">
  import * as api from "$lib/api";
  import { app, toast } from "$lib/stores.svelte";
  import Modal from "./Modal.svelte";

  // Mirrors DEFAULT_JUNK_PHRASES in clip2pod-core/src/junk.rs.
  const DEFAULTS = [
    "credit:",
    "getty images",
    "http",
    "listen to article",
    "min read",
    "read more",
    "read this article for free",
    "related links",
    "related stories",
    "unsplash",
    "view image in full size",
    "view original",
  ];

  let text = $state("");

  $effect(() => {
    api.getJunkPhrases().then((phrases) => (text = phrases.join("\n")));
  });

  async function save() {
    const phrases = text
      .split("\n")
      .map((p) => p.trim())
      .filter(Boolean);
    await api.setJunkPhrases(phrases);
    app.dialog = null;
    toast("Junk phrases saved");
  }
</script>

<Modal title="Junk phrases" onclose={() => (app.dialog = null)}>
  <p class="hint">
    One phrase per line. Find Junk flags any line containing one of these
    (case-insensitive).
  </p>
  <textarea class="field phrases" bind:value={text} spellcheck="false"></textarea>

  {#snippet footer()}
    <button class="btn" onclick={() => (text = DEFAULTS.join("\n"))}>Restore defaults</button>
    <button class="btn primary" onclick={save}>Save</button>
  {/snippet}
</Modal>

<style>
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 0 0 10px;
  }

  .phrases {
    width: 100%;
    height: 42vh;
    resize: none;
    font-family: var(--mono);
    font-size: 12.5px;
    line-height: 1.7;
  }
</style>
