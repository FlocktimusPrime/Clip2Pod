<script lang="ts">
  import { app, loadVoices } from "$lib/stores.svelte";

  let {
    onpaste,
    onfind,
    onclean,
    ongenerate,
    onfetch,
  }: {
    onpaste: () => void;
    onfind: () => void;
    onclean: () => void;
    ongenerate: () => void;
    onfetch: (url: string) => Promise<void>;
  } = $props();

  let url = $state("");
  let fetching = $state(false);

  async function fetchUrl() {
    const target = url.trim();
    if (!target || fetching) return;
    fetching = true;
    try {
      await onfetch(target);
      url = "";
    } finally {
      fetching = false;
    }
  }

  const canGenerate = $derived(app.voices !== null && !app.voicesError);
</script>

<aside class="side">
  <div class="group">
    <span class="label">Intake</span>
    <div class="folder-row">
      <input
        class="field"
        placeholder="Article URL…"
        bind:value={url}
        onkeydown={(e) => e.key === "Enter" && fetchUrl()}
      />
      <button class="btn" onclick={fetchUrl} disabled={fetching || !url.trim()}>
        {fetching ? "…" : "Fetch"}
      </button>
    </div>
  </div>

  <div class="group">
    <span class="label">Transport</span>
    <button class="btn" onclick={onpaste}>Paste clipboard <kbd>Ctrl+Shift+V</kbd></button>
    <button class="btn" onclick={onfind}>Find junk <kbd>Ctrl+F</kbd></button>
    <button class="btn" onclick={onclean}>Clean for TTS <kbd>Ctrl+L</kbd></button>
    <button class="btn primary" onclick={ongenerate} disabled={!canGenerate}>
      Generate MP3 <kbd>Ctrl+Enter</kbd>
    </button>
  </div>

  <div class="group">
    <span class="label">Tools</span>
    <button class="btn" onclick={() => (app.dialog = "voices")}>Manage voices <kbd>Ctrl+M</kbd></button>
    <button class="btn" onclick={() => (app.dialog = "junk")}>Junk phrases <kbd>Ctrl+J</kbd></button>
    <button class="btn" onclick={() => (app.dialog = "queue")}>Queue <kbd>Ctrl+Q</kbd></button>
    <button class="btn" onclick={() => (app.dialog = "log")}>Log <kbd>Ctrl+Shift+L</kbd></button>
    <button class="btn" onclick={() => (app.dialog = "settings")}>Settings</button>
  </div>

  <div class="group">
    <span class="label">Voices enabled</span>
    {#if app.voices}
      <p class="mono-line">
        {app.voices.male_enabled}/{app.voices.male_total} male ·
        {app.voices.female_enabled}/{app.voices.female_total} female
      </p>
    {:else if app.voicesLoading}
      <p class="mono-line muted">loading catalog…</p>
    {:else}
      <p class="mono-line muted">unavailable</p>
    {/if}
  </div>

  {#if app.voicesError}
    <div class="banner" role="alert">
      <p>Voice list unavailable: {app.voicesError}</p>
      <button class="btn" onclick={() => loadVoices(true)} disabled={app.voicesLoading}>
        {app.voicesLoading ? "Retrying…" : "Retry"}
      </button>
    </div>
  {/if}
</aside>

<style>
  .side {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px;
    background: var(--panel);
    border-left: 1px solid var(--line);
    width: 240px;
    overflow-y: auto;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .group > .label {
    margin-bottom: 2px;
  }

  .side .btn {
    padding: 5px 10px;
  }

  .folder-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .folder-row .btn {
    flex-shrink: 0;
  }

  .mono-line {
    font-family: var(--mono);
    font-size: 11px;
    margin: 0;
    color: var(--text);
  }

  .mono-line.muted {
    color: var(--muted);
  }

  .banner {
    border: 1px solid var(--danger);
    border-radius: 4px;
    padding: 10px;
    font-size: 12px;
    background: color-mix(in srgb, var(--danger) 10%, transparent);
  }

  .banner p {
    margin: 0 0 8px;
    overflow-wrap: anywhere;
  }
</style>
