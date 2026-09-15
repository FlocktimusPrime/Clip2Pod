// Central UI state (Svelte 5 runes) plus the Tauri event subscriptions that
// keep it live. Everything the backend pushes lands here; components only read.

import { listen } from "@tauri-apps/api/event";
import * as api from "./api";
import * as rip from "./rip_api";
import type {
  ConfigView,
  DoctorReport,
  Job,
  Lamp,
  LogEntry,
  RenderProgress,
  RipConfigView,
  RipEpisodeView,
  RipJob,
  RipJobProgress,
  RipLogEntry,
  Theme,
  VoicesView,
} from "./types";

export type TabName = "narrate" | "rip";

export type DialogName =
  | "voices"
  | "authors"
  | "junk"
  | "log"
  | "rip-log"
  | "feed"
  | "settings"
  | "startup-prompt"
  | null;

export const app = $state({
  /** Which mode tab is showing. */
  tab: "narrate" as TabName,

  // --- NARRATE ---
  lamp: { state: "Idle" } as Lamp,
  queue: [] as Job[],
  /** Chunk progress of the job rendering now; null between jobs. */
  renderProgress: null as RenderProgress | null,
  /** Newest-first, mirrors log.json; dialogs read this without refetching. */
  log: [] as LogEntry[],
  voices: null as VoicesView | null,
  /** Non-empty = fetch failed and no cache: banner shown, generation disabled. */
  voicesError: "",
  voicesLoading: false,
  config: null as ConfigView | null,

  // --- RIP ---
  ripLamp: { state: "Idle" } as Lamp,
  ripQueue: [] as RipJob[],
  ripEpisodes: [] as RipEpisodeView[],
  ripLog: [] as RipLogEntry[],
  ripConfig: null as RipConfigView | null,
  /** Preflight: null until probed; the RIP tab shows a notice when a tool is missing. */
  doctor: null as DoctorReport | null,

  dialog: null as DialogName,
  toasts: [] as { id: number; text: string; kind: "info" | "error" }[],
});

let toastSeq = 0;

export function toast(text: string, kind: "info" | "error" = "info") {
  const id = ++toastSeq;
  app.toasts.push({ id, text, kind });
  setTimeout(() => {
    app.toasts = app.toasts.filter((t) => t.id !== id);
  }, 4200);
}

export function applyTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme;
}

export async function loadVoices(refresh: boolean) {
  app.voicesLoading = true;
  app.voicesError = "";
  try {
    app.voices = await api.listVoices(refresh);
  } catch (e) {
    app.voicesError = String(e);
  } finally {
    app.voicesLoading = false;
  }
}

export async function refreshRipEpisodes() {
  try {
    app.ripEpisodes = await rip.listEpisodes();
  } catch (e) {
    toast(`Could not list ripped episodes: ${e}`, "error");
  }
}

/** Re-probe for yt-dlp / ffmpeg. Non-fatal — the RIP tab just hides the notice. */
export async function refreshDoctor() {
  try {
    app.doctor = await rip.doctor();
  } catch (e) {
    console.error("rip doctor failed", e);
  }
}

/** One-time startup: config, theme, voice catalog, event subscriptions. */
export async function initApp() {
  await listen<Lamp>("lamp", (e) => (app.lamp = e.payload));
  await listen<Job[]>("queue-changed", (e) => {
    app.queue = e.payload;
    // No job processing → clear any stale progress so the footer stops showing %.
    if (!e.payload.some((j) => j.status === "Processing")) app.renderProgress = null;
  });
  await listen<RenderProgress>("render-progress", (e) => (app.renderProgress = e.payload));
  await listen<LogEntry>("log-appended", (e) => app.log.unshift(e.payload));
  await listen<VoicesView>("voices-changed", (e) => (app.voices = e.payload));

  // RIP mode events (namespaced so they don't collide with the narrate worker's).
  await listen<Lamp>("rip:lamp", (e) => (app.ripLamp = e.payload));
  await listen<RipJob[]>("rip:queue-changed", (e) => {
    const wasRunning = app.ripQueue.some((j) => j.status === "Processing");
    app.ripQueue = e.payload;
    const running = app.ripQueue.some((j) => j.status === "Processing");
    // a rip job just finished — the ripped-audio folder changed
    if (wasRunning && !running) void refreshRipEpisodes();
  });
  await listen<{ id: string; progress: RipJobProgress }>("rip:job-progress", (e) => {
    const job = app.ripQueue.find((j) => j.id === e.payload.id);
    if (job) job.progress = e.payload.progress;
  });
  await listen<RipLogEntry>("rip:log-appended", (e) => app.ripLog.unshift(e.payload));

  // A capture (extension or hotkey) tells the UI which mode to show.
  await listen<TabName>("switch-tab", (e) => (app.tab = e.payload));

  app.config = await api.getConfig();
  applyTheme(app.config.theme);
  app.queue = await api.getQueue();
  app.log = await api.getLog("");
  await loadVoices(false);
  if (app.config.launch_at_startup === null) app.dialog = "startup-prompt";

  app.ripConfig = await rip.getConfig();
  app.ripQueue = await rip.getQueue();
  app.ripLog = await rip.getLog("");
  await refreshRipEpisodes();
  void refreshDoctor();
}
