<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/api";
  import { app, applyTheme } from "$lib/stores.svelte";
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

  async function toggleLaunchAtStartup(e: Event) {
    if (!app.config) return;
    const on = (e.currentTarget as HTMLInputElement).checked;
    app.config.launch_at_startup = on;
    await api.setLaunchAtStartup(on);
  }

  async function switchTheme(theme: Theme) {
    if (app.config) app.config.theme = theme;
    applyTheme(theme);
    await api.setTheme(theme);
  }
</script>

<Modal title="Settings" onclose={() => (app.dialog = null)}>
  <div class="section">
    <h3 class="label">Episode folder</h3>
    <div class="row">
      <p class="path mono" title={app.config?.output_dir}>{app.config?.output_dir ?? "…"}</p>
      <button class="btn" onclick={pickOutputDir}>Browse</button>
    </div>
    <p class="hint">The feed serves every .mp3 in this folder; cover.jpg here is the artwork.</p>
    <label class="check">
      <input type="checkbox" checked={app.config?.prefix_c2p ?? false} onchange={togglePrefix} />
      <span>Prefix filenames with C2P</span>
    </label>
  </div>

  <div class="section">
    <h3 class="label">Startup</h3>
    <label class="check">
      <input
        type="checkbox"
        checked={app.config?.start_minimized ?? false}
        onchange={toggleStartMinimized}
      />
      <span>Start minimized to tray</span>
    </label>
    <label class="check">
      <input
        type="checkbox"
        checked={app.config?.launch_at_startup ?? false}
        onchange={toggleLaunchAtStartup}
      />
      <span>Launch Clip2Pod when you sign in</span>
    </label>
  </div>

  <div class="section">
    <h3 class="label">Theme</h3>
    <div class="row">
      <button
        class="btn"
        class:primary={app.config?.theme === "dark"}
        aria-pressed={app.config?.theme === "dark"}
        onclick={() => switchTheme("dark")}>Dark</button
      >
      <button
        class="btn"
        class:primary={app.config?.theme === "light"}
        aria-pressed={app.config?.theme === "light"}
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
</style>
