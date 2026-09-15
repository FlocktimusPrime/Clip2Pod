// TypeScript mirrors of the serde types in clip2pod-core / the Tauri shell.
// Field names are snake_case because the Rust structs have no rename_all.

export type Gender = "Male" | "Female";
export type AuthorGender = "Unknown" | "Male" | "Female";
export type Theme = "dark" | "light";

export interface VoiceInfo {
  name: string;
  short_name: string;
  gender: Gender;
  locale: string;
  language: string;
  country: string;
  category: string;
}

export interface VoicesView {
  voices: VoiceInfo[];
  enabled: string[];
  male_total: number;
  female_total: number;
  male_enabled: number;
  female_enabled: number;
}

export type JobStatus = "Queued" | "Processing" | "Done" | "Failed" | "Cancelled";

export interface Job {
  id: string;
  title: string;
  author: string;
  text: string;
  voice: VoiceInfo;
  filename: string;
  status: JobStatus;
  created: string;
  started: string | null;
  finished: string | null;
  detail: string;
}

export interface RenderProgress {
  done: number;
  total: number;
}

export type LogStatus = "Done" | "Failed" | "Cancelled";

export interface LogEntry {
  timestamp: string;
  status: LogStatus;
  title: string;
  voice: string;
  filename: string;
  detail: string;
}

export interface Autofill {
  title: string;
  author: string;
  filename_title: string;
}

export interface CleanResult {
  cleaned: string;
  autofill: Autofill;
}

export interface JunkMatch {
  line_idx: number;
  match_start: number;
  match_end: number;
  phrase: string;
}

export interface AuthorEntry {
  name: string;
  gender: AuthorGender;
}

export interface ConfigView {
  output_dir: string;
  prefix_c2p: boolean;
  author_gender: AuthorGender;
  theme: Theme;
  start_minimized: boolean;
  launch_at_startup: boolean | null;
}

export interface Extracted {
  title: string;
  author: string;
  text: string;
}

export interface FirewallHelp {
  shell_hint: string;
  command: string;
  tips: string[];
}

export interface CapturedArticle extends Extracted {
  url: string;
}

export type Lamp =
  | { state: "Idle" }
  | { state: "Queued"; queued: number }
  | { state: "OnAir" };

// --- RIP mode (yt-dlp). Mirrors ytdlfeed-core / rip_commands.rs. The narrate
// types above keep their bare names (this is the host app); rip types are
// prefixed. `Lamp` and `Theme` are shared — identical shape on both sides. ---

export type RipJobStatus = JobStatus;

export interface RipJobProgress {
  percent: number | null;
  stage: string | null;
  speed: string | null;
}

export interface RipJob {
  id: string;
  url: string;
  filename: string | null;
  status: RipJobStatus;
  progress: RipJobProgress;
  created: string;
  started: string | null;
  finished: string | null;
  detail: string;
}

export type RipLogStatus = "Queued" | "Done" | "Failed" | "Cancelled";

export interface RipLogEntry {
  timestamp: string;
  status: RipLogStatus;
  title: string;
  detail: string;
}

export interface RipConfigView {
  output_dir: string;
  args_template: string;
  default_args: string;
  /** Configured yt-dlp binary, or "" meaning "yt-dlp from PATH". */
  ytdlp_path: string;
}

export interface RipEpisodeView {
  title: string;
  filename: string;
  size: number;
  modified: string;
  artist: string | null;
  duration_ms: number | null;
}

export interface DoctorReport {
  ytdlp_found: boolean;
  ytdlp_version: string | null;
  ffmpeg_found: boolean;
}
