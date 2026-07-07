<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/api";
  import { app, loadVoices, toast } from "$lib/stores.svelte";

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

  async function pickOutputDir() {
    const dir = await open({ directory: true, title: "Choose output folder" });
    if (typeof dir === "string" && app.config) {
      await api.setOutputDir(dir);
      app.config.output_dir = dir;
    }
  }

  async function togglePrefix(e: Event) {
    if (!app.config) return;
    const on = (e.currentTarget as HTMLInputElement).checked;
    app.config.prefix_c2p = on;
    await api.setPrefix(on);
  }

  /** Episode count pending delete confirmation; null = no confirm active. */
  let confirmCount = $state<number | null>(null);

  async function askDeleteEpisodes() {
    try {
      const count = await api.episodeCount();
      if (count === 0) {
        toast("No episodes in output folder");
        return;
      }
      confirmCount = count;
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function confirmDeleteEpisodes() {
    try {
      const deleted = await api.deleteEpisodes();
      toast(`Deleted ${deleted} episode${deleted === 1 ? "" : "s"}`);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      confirmCount = null;
    }
  }
</script>

<aside class="side">
  <div class="rack">
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

  <div class="rack">
    <span class="label">Transport</span>
    <button class="btn" onclick={onpaste}>Paste clipboard <kbd>Ctrl+Shift+V</kbd></button>
    <button class="btn" onclick={onfind}>Find junk <kbd>Ctrl+F</kbd></button>
    <button class="btn" onclick={onclean}>Clean for TTS <kbd>Ctrl+L</kbd></button>
    <button class="btn primary" onclick={ongenerate} disabled={!canGenerate}>
      Generate MP3 <kbd>Ctrl+Enter</kbd>
    </button>
  </div>

  <div class="rack">
    <span class="label">Output</span>
    <div class="folder-row">
      <p class="mono-line path" title={app.config?.output_dir}>{app.config?.output_dir ?? "…"}</p>
      <button class="btn" onclick={pickOutputDir}>Change</button>
    </div>
    <label class="check">
      <input type="checkbox" checked={app.config?.prefix_c2p ?? false} onchange={togglePrefix} />
      <span>Prefix filenames with C2P</span>
    </label>
    {#if confirmCount === null}
      <button class="btn" onclick={askDeleteEpisodes}>Delete episodes</button>
    {:else}
      <div class="folder-row confirm" role="alertdialog" aria-label="Confirm episode deletion">
        <span>Delete {confirmCount} episode{confirmCount === 1 ? "" : "s"}?</span>
        <button class="btn" onclick={confirmDeleteEpisodes}>Confirm</button>
        <button class="btn" onclick={() => (confirmCount = null)}>Cancel</button>
      </div>
    {/if}
  </div>

  <div class="rack">
    <span class="label">Desk</span>
    <button class="btn" onclick={() => (app.dialog = "voices")}>Manage voices <kbd>Ctrl+M</kbd></button>
    <button class="btn" onclick={() => (app.dialog = "junk")}>Junk phrases <kbd>Ctrl+J</kbd></button>
    <button class="btn" onclick={() => (app.dialog = "queue")}>Queue <kbd>Ctrl+Q</kbd></button>
    <button class="btn" onclick={() => (app.dialog = "log")}>Log <kbd>Ctrl+Shift+L</kbd></button>
  </div>

  <div class="rack">
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

  .rack {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .rack > .label {
    margin-bottom: 2px;
  }

  .side .btn {
    padding: 5px 10px;
  }

  .btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
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

  .path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .confirm {
    flex-wrap: wrap;
    font-size: 12px;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    cursor: pointer;
  }

  .check input {
    accent-color: var(--amber);
  }

  .banner {
    border: 1px solid var(--onair);
    border-radius: 4px;
    padding: 10px;
    font-size: 12px;
    background: color-mix(in srgb, var(--onair) 10%, transparent);
  }

  .banner p {
    margin: 0 0 8px;
    overflow-wrap: anywhere;
  }
</style>
