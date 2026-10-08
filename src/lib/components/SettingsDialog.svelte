<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "$lib/api";
  import * as rip from "$lib/rip_api";
  import { app, applyTheme, refreshDoctor, refreshRipEpisodes, toast } from "$lib/stores.svelte";
  import type { Theme } from "$lib/types";
  import Modal from "./Modal.svelte";

  // Shown at the bottom so bug reports can quote it.
  let version = $state("");
  getVersion().then((v) => (version = v));

  // --- General ---
  async function switchTheme(theme: Theme) {
    if (app.config) app.config.theme = theme;
    applyTheme(theme);
    await api.setTheme(theme);
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

  // --- Narrate ---
  async function pickNarrateDir() {
    const dir = await open({ directory: true, title: "Choose narrated-audio folder" });
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

  // --- Rip ---
  let ripArgs = $state(app.ripConfig?.args_template ?? "");
  let ripBin = $state(app.ripConfig?.ytdlp_path ?? "");
  let ripArgsError = $state("");

  const ripDirty = $derived(
    app.ripConfig !== null &&
      (ripArgs.trim() !== app.ripConfig.args_template || ripBin.trim() !== app.ripConfig.ytdlp_path),
  );

  async function pickRipDir() {
    const dir = await open({ directory: true, title: "Choose ripped-audio folder" });
    if (typeof dir === "string" && app.ripConfig) {
      await rip.setOutputDir(dir);
      app.ripConfig.output_dir = dir;
    }
  }

  function resetRipArgs() {
    if (app.ripConfig) ripArgs = app.ripConfig.default_args;
  }

  async function saveRip() {
    if (!app.ripConfig) return;
    ripArgsError = "";
    try {
      await rip.setArgsTemplate(ripArgs);
    } catch (e) {
      ripArgsError = String(e);
      return;
    }
    await rip.setYtdlpPath(ripBin);
    app.ripConfig = await rip.getConfig();
    ripArgs = app.ripConfig.args_template;
    ripBin = app.ripConfig.ytdlp_path;
    await refreshDoctor();
    await refreshRipEpisodes();
    toast("Rip settings saved");
  }
</script>

<Modal title="Settings" onclose={() => (app.dialog = null)}>
  <h3 class="section-head label">General</h3>
  <div class="section">
    <div class="row" role="group" aria-label="Theme">
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

  <h3 class="section-head label">Narrate</h3>
  <div class="section">
    <span class="label">Narrated-audio folder</span>
    <div class="row">
      <p class="path mono" title={app.config?.output_dir}>{app.config?.output_dir ?? "…"}</p>
      <button class="btn" aria-label="Browse for the narrated-audio folder" onclick={pickNarrateDir}>
        Browse
      </button>
    </div>
    <label class="check">
      <input type="checkbox" checked={app.config?.prefix_c2p ?? false} onchange={togglePrefix} />
      <span>Prefix filenames with C2P</span>
    </label>
    <button class="btn" onclick={() => (app.dialog = "voices")}>Manage voices</button>
    <button class="btn" onclick={() => (app.dialog = "authors")}>Manage Author Genders</button>
  </div>

  <h3 class="section-head label">Rip</h3>
  <div class="section">
    <span class="label">Ripped-audio folder</span>
    <div class="row">
      <p class="path mono" title={app.ripConfig?.output_dir}>{app.ripConfig?.output_dir ?? "…"}</p>
      <button class="btn" aria-label="Browse for the ripped-audio folder" onclick={pickRipDir}>
        Browse
      </button>
    </div>

    <label class="label" for="rip-args">yt-dlp arguments</label>
    <textarea
      id="rip-args"
      class="field args"
      rows="5"
      bind:value={ripArgs}
      spellcheck="false"
      aria-invalid={ripArgsError ? "true" : undefined}
      aria-describedby={ripArgsError ? "rip-args-error rip-args-hint" : "rip-args-hint"}
    ></textarea>
    {#if ripArgsError}
      <p class="error" id="rip-args-error">{ripArgsError}</p>
    {/if}
    <p class="hint" id="rip-args-hint">
      The app always appends the output folder and the video URL. Output must stay mp3 for the
      feed to list it.
    </p>
    <button class="btn" onclick={resetRipArgs}>Reset to default</button>

    <label class="label" for="rip-bin">yt-dlp binary</label>
    <input
      id="rip-bin"
      class="field mono"
      bind:value={ripBin}
      spellcheck="false"
      placeholder="yt-dlp (from PATH)"
      aria-describedby="rip-bin-hint"
    />
    <p class="hint" id="rip-bin-hint">
      Leave blank to use <code>yt-dlp</code> from your PATH, or point at a specific build (e.g. a
      nightly) if rips start failing.
    </p>
    {#if app.doctor?.ytdlp_version}
      <p class="hint">
        Detected: yt-dlp {app.doctor.ytdlp_version}{app.doctor.ffmpeg_found ? "" : " · ffmpeg NOT found"}
      </p>
    {:else if app.doctor}
      <p class="error">yt-dlp not found — see the notice on the Rip tab.</p>
    {/if}

    <button class="btn primary" onclick={saveRip} disabled={!ripDirty}>Save rip settings</button>
  </div>

  {#if version}
    <p class="hint mono version">Clip2Pod {version}</p>
  {/if}
</Modal>

<style>
  /* Type comes from .label (muted mono, per DESIGN.md settings-subsection spec);
     this only sets the rhythm — more space above the heading than below. */
  .section-head {
    margin: 4px 0 10px;
  }

  .section-head:not(:first-child) {
    margin-top: 20px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
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

  .args {
    font-family: var(--mono);
    font-size: 12px;
    line-height: 1.5;
    resize: vertical;
    width: 100%;
  }

  .field.mono {
    width: 100%;
  }

  .hint {
    font-size: 12px;
    color: var(--muted);
    margin: 0;
  }

  .version {
    margin-top: 20px;
    user-select: text;
  }

  .error {
    font-size: 12px;
    color: var(--danger);
    margin: 0;
  }
</style>
