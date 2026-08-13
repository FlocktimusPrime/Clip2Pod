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

export interface CapturedArticle extends Extracted {
  url: string;
}

export type Lamp =
  | { state: "Idle" }
  | { state: "Queued"; queued: number }
  | { state: "OnAir" };
