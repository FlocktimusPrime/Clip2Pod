// Thin typed wrappers over the Tauri IPC commands in src-tauri/src/commands.rs.
// Argument keys are camelCase; Tauri 2 maps them to the snake_case Rust params.

import { invoke } from "@tauri-apps/api/core";
import type {
  AuthorGender,
  CleanResult,
  ConfigView,
  Extracted,
  FirewallHelp,
  Job,
  JunkMatch,
  LogEntry,
  Theme,
  VoicesView,
} from "./types";

export const cleanText = (text: string) => invoke<CleanResult>("clean_text", { text });

export const findJunk = (text: string, fromLine: number) =>
  invoke<JunkMatch | null>("find_junk", { text, fromLine });

export const getJunkPhrases = () => invoke<string[]>("get_junk_phrases");

export const setJunkPhrases = (phrases: string[]) => invoke<void>("set_junk_phrases", { phrases });

export const defaultJunkPhrases = () => invoke<string[]>("default_junk_phrases");

export const suggestJunk = (text: string) => invoke<string[]>("suggest_junk", { text });

export const listVoices = (refresh: boolean) => invoke<VoicesView>("list_voices", { refresh });

export const setVoiceEnabled = (shortName: string, enabled: boolean) =>
  invoke<void>("set_voice_enabled", { shortName, enabled });

export const enableAllVoices = () => invoke<void>("enable_all_voices");

export const disableAllVoices = () => invoke<void>("disable_all_voices");

export const previewVoice = (shortName: string) =>
  invoke<ArrayBuffer>("preview_voice", { shortName });

export const enqueueGenerate = (
  text: string,
  title: string,
  author: string,
  filenameTitle: string,
  sourceUrl: string | null,
) => invoke<string>("enqueue_generate", { text, title, author, filenameTitle, sourceUrl });

export const getQueue = () => invoke<Job[]>("get_queue");

export const clearPending = () => invoke<void>("clear_pending");

export const cancelCurrent = () => invoke<void>("cancel_current");

export const getLog = (query: string) => invoke<LogEntry[]>("get_log", { query });

export const clearLog = () => invoke<void>("clear_log");

export const getConfig = () => invoke<ConfigView>("get_config");

export const setOutputDir = (dir: string) => invoke<void>("set_output_dir", { dir });

export const setPrefix = (prefix: boolean) => invoke<void>("set_prefix", { prefix });

export const setAuthorGender = (gender: AuthorGender) =>
  invoke<void>("set_author_gender", { gender });

export const setTheme = (theme: Theme) => invoke<void>("set_theme", { theme });

export const setStartMinimized = (minimized: boolean) =>
  invoke<void>("set_start_minimized", { minimized });

export const setLaunchAtStartup = (enabled: boolean) =>
  invoke<void>("set_launch_at_startup", { enabled });

export const extractUrl = (url: string) => invoke<Extracted>("extract_url", { url });

export const feedUrl = () => invoke<string>("feed_url");

export const episodeCount = () => invoke<number>("episode_count");

export const deleteAllEpisodes = () => invoke<number>("delete_all_episodes");

export const firewallHelp = () => invoke<FirewallHelp>("firewall_help");
