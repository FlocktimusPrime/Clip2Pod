// Central UI state (Svelte 5 runes) plus the Tauri event subscriptions that
// keep it live. Everything the backend pushes lands here; components only read.

import { listen } from "@tauri-apps/api/event";
import * as api from "./api";
import type { ConfigView, Job, Lamp, LogEntry, Theme, VoicesView } from "./types";

export type DialogName =
  | "voices"
  | "junk"
  | "queue"
  | "log"
  | "feed"
  | "settings"
  | "startup-prompt"
  | null;

export const app = $state({
  lamp: { state: "Idle" } as Lamp,
  queue: [] as Job[],
  /** Newest-first, mirrors log.json; dialogs read this without refetching. */
  log: [] as LogEntry[],
  voices: null as VoicesView | null,
  /** Non-empty = fetch failed and no cache: banner shown, generation disabled. */
  voicesError: "",
  voicesLoading: false,
  config: null as ConfigView | null,
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

/** One-time startup: config, theme, voice catalog, event subscriptions. */
export async function initApp() {
  await listen<Lamp>("lamp", (e) => (app.lamp = e.payload));
  await listen<Job[]>("queue-changed", (e) => (app.queue = e.payload));
  await listen<LogEntry>("log-appended", (e) => app.log.unshift(e.payload));
  await listen<VoicesView>("voices-changed", (e) => (app.voices = e.payload));

  app.config = await api.getConfig();
  applyTheme(app.config.theme);
  app.queue = await api.getQueue();
  app.log = await api.getLog("");
  await loadVoices(false);
  if (app.config.launch_at_startup === null) app.dialog = "startup-prompt";
}
