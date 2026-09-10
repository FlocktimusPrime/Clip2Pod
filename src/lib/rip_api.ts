// Typed wrappers over the rip_* Tauri IPC commands (src-tauri/src/rip_commands.rs).
// Argument keys are camelCase; Tauri 2 maps them to the snake_case Rust params.
// Kept separate from api.ts (the narrate commands) — import as `* as rip`.

import { invoke } from "@tauri-apps/api/core";
import type { DoctorReport, RipConfigView, RipEpisodeView, RipJob, RipLogEntry } from "./types";

export const enqueue = (url: string) => invoke<string>("rip_enqueue", { url });

export const getQueue = () => invoke<RipJob[]>("rip_get_queue");

export const clearPending = () => invoke<void>("rip_clear_pending");

export const stopJob = (jobId: string) => invoke<void>("rip_stop_job", { jobId });

export const getLog = (query: string) => invoke<RipLogEntry[]>("rip_get_log", { query });

export const clearLog = () => invoke<void>("rip_clear_log");

export const getConfig = () => invoke<RipConfigView>("rip_get_config");

export const setOutputDir = (dir: string) => invoke<void>("rip_set_output_dir", { dir });

export const setArgsTemplate = (template: string) =>
  invoke<void>("rip_set_args_template", { template });

export const setYtdlpPath = (path: string) => invoke<void>("rip_set_ytdlp_path", { path });

export const listEpisodes = () => invoke<RipEpisodeView[]>("rip_list_episodes");

export const deleteEpisode = (filename: string) =>
  invoke<void>("rip_delete_episode", { filename });

export const deleteAllEpisodes = () => invoke<number>("rip_delete_all_episodes");

export const feedUrl = () => invoke<string>("rip_feed_url");

export const doctor = () => invoke<DoctorReport>("rip_doctor");
