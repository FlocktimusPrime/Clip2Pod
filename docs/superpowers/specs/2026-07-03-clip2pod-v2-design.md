# Clip2Pod v2 — Design

Date: 2026-07-03
Status: approved

## Goal

Fresh rewrite of Clip2Pod (clipboard text → cleaned script → Edge TTS narration → ID3-tagged MP3) that runs on Windows and Linux. Feature parity with the v1 spec (`ABOUT.md`), plus two additions chosen during design interview: system tray + global hotkey, and URL article extraction.

## Decisions

| Decision | Choice |
|---|---|
| Stack | Tauri 2 (Rust backend) + Svelte 5 + TypeScript + Vite |
| Architecture | Fat Rust core: all logic in a Tauri-free library crate (`clip2pod-core`); Svelte is a pure view layer |
| Improvements | Tray + global hotkey (default `Ctrl+Alt+G`); URL → readability extraction |
| Packaging | Windows portable .exe + Linux AppImage (GitHub Actions builds Windows) |
| UI | Broadcast "production desk" identity: ON AIR lamp, studio dark default, daylight light theme |
| v1 config | No migration — fresh start |

## Architecture

```
src/                          Svelte frontend (view only)
src-tauri/src/                Tauri shell: IPC commands, events, TTS worker, tray, global shortcut
src-tauri/crates/clip2pod-core/   pure logic, unit-tested with cargo test
```

Key dependencies: `msedge-tts` (voice list + MP3 synthesis), `dom_smoothie` (readability), `id3` (tagging), `reqwest`, `dirs`; Tauri plugins: clipboard-manager, global-shortcut, dialog, notification; tray built into Tauri 2.

## Core modules

- **textclean** — smart punctuation→ASCII, emoji/pictograph strip, decorative symbol removal, symbol-run strip, whitespace collapse; autofill: title = line 1, author = shorter of lines 2/3, filename-title = line 1.
- **junk** — 12 default phrases, case-insensitive substring, `find_next` without auto-wrap; custom list persisted only when it differs from defaults.
- **voices** — English non-cartoon filter; default-enabled = standard `en-US` minus Multilingual variants; gender pools; Unknown alternates gender; round-robin per pool; cycling state persisted.
- **queue** — jobs Queued/Processing/Done/Failed/Cancelled; single worker; Clear Pending never touches the in-flight job.
- **naming** — Windows-safe sanitize on both OSes, trailing dot/space trim, length cap, optional `C2P_` prefix, `(2)/(3)` collision suffix vs disk and queue.
- **tagging** — ID3v2 Title/Artist/Album="Clip2Pod"/Comment=voice; best-effort.
- **config** — JSON at platform config dir `Clip2Pod/config.json`; log.json capped 200; atomic writes; crash.log panic hook.
- **extract** — reqwest + dom_smoothie → plain text.

## Data flow (Generate)

Frontend → `enqueue_generate` → core cleans, picks voice (gender pref / alternation / round-robin), reserves collision-safe filename, queues → worker synthesizes → writes MP3 → tags → logs. Events: `lamp` (Idle/Queued n/OnAir), `queue-changed`, `log-appended`.

## Error handling

- Voice list fetch failure: cached list fallback; none → error banner + retry, generation disabled.
- TTS failure: job Failed with detail, worker continues.
- Empty clipboard: warn, editor untouched.
- URL extraction failure: toast, editor untouched.
- Tagging failure: logged, MP3 kept.
- Panic hook → crash.log.

## Testing

`cargo test` covers all core modules (cleaning cases, junk matching, autofill heuristics, cycling determinism across simulated restarts, filename collisions, config roundtrip). Real-network TTS test behind `--ignored`. Frontend verified manually via `tauri dev`; packaging verified by building AppImage locally and Windows .exe in CI.

## Known limitations

- Global hotkey may not register under Linux Wayland; tray menu is the fallback.
- Edge TTS is an unofficial endpoint; if `msedge-tts` breaks against upstream changes, fallback is porting its websocket handshake in-repo.
