<script lang="ts">
  import * as api from "$lib/api";
  import { app, toast } from "$lib/stores.svelte";
  import Modal from "./Modal.svelte";

  let text = $state("");
  let defaults = $state<string[]>([]);

  $effect(() => {
    api.getJunkPhrases().then((phrases) => (text = phrases.join("\n")));
    api.defaultJunkPhrases().then((phrases) => (defaults = phrases));
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
    (case-insensitive). Use <code>*</code> as a wildcard, e.g.
    <code>photo*getty</code>.
  </p>
  <textarea
    class="field phrases"
    aria-label="Junk phrases, one per line"
    bind:value={text}
    spellcheck="false"
  ></textarea>

  {#snippet footer()}
    <button class="btn" onclick={() => (text = defaults.join("\n"))} disabled={defaults.length === 0}>
      Restore defaults
    </button>
    <button class="btn primary" onclick={save}>Save</button>
  {/snippet}
</Modal>

<style>
  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 0 0 10px;
  }

  .hint code {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text);
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
