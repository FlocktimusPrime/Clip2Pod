<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/api";
  import { app, applyTheme, toast } from "$lib/stores.svelte";
  import type { Theme } from "$lib/types";
  import Modal from "./Modal.svelte";

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

  async function toggleStartMinimized(e: Event) {
    if (!app.config) return;
    const on = (e.currentTarget as HTMLInputElement).checked;
    app.config.start_minimized = on;
    await api.setStartMinimized(on);
  }

  async function switchTheme(theme: Theme) {
    if (app.config) app.config.theme = theme;
    applyTheme(theme);
    await api.setTheme(theme);
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
      const deleted = await api.deleteAllEpisodes();
      toast(`Deleted ${deleted} episode${deleted === 1 ? "" : "s"}`);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      confirmCount = null;
    }
  }
</script>

<Modal title="Settings" onclose={() => (app.dialog = null)}>
  <div class="section">
    <span class="label">Episode folder</span>
    <div class="row">
      <p class="path mono" title={app.config?.output_dir}>{app.config?.output_dir ?? "…"}</p>
      <button class="btn" onclick={pickOutputDir}>Browse</button>
    </div>
    <p class="hint">The feed serves every .mp3 in this folder; cover.jpg here is the artwork.</p>
    <label class="check">
      <input type="checkbox" checked={app.config?.prefix_c2p ?? false} onchange={togglePrefix} />
      <span>Prefix filenames with C2P</span>
    </label>
    {#if confirmCount === null}
      <button class="btn" onclick={askDeleteEpisodes}>Delete episodes</button>
    {:else}
      <div class="row confirm" role="alertdialog" aria-label="Confirm episode deletion">
        <span>Delete {confirmCount} episode{confirmCount === 1 ? "" : "s"}?</span>
        <button class="btn" onclick={confirmDeleteEpisodes}>Confirm</button>
        <button class="btn" onclick={() => (confirmCount = null)}>Cancel</button>
      </div>
    {/if}
  </div>

  <div class="section">
    <span class="label">Startup</span>
    <label class="check">
      <input
        type="checkbox"
        checked={app.config?.start_minimized ?? false}
        onchange={toggleStartMinimized}
      />
      <span>Start minimized to tray</span>
    </label>
  </div>

  <div class="section">
    <span class="label">Theme</span>
    <div class="row">
      <button
        class="btn"
        class:primary={app.config?.theme === "dark"}
        onclick={() => switchTheme("dark")}>Dark</button
      >
      <button
        class="btn"
        class:primary={app.config?.theme === "light"}
        onclick={() => switchTheme("light")}>Light</button
      >
    </div>
  </div>
</Modal>

<style>
  .section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 18px;
    align-items: flex-start;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
  }

  .path {
    flex: 1;
    min-width: 0;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mono {
    font-family: var(--mono);
    font-size: 11.5px;
  }

  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 0;
  }

  .confirm {
    flex-wrap: wrap;
    font-size: 12px;
  }
</style>
