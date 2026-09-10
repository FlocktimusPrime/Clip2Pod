# CLAUDE.md

Guidance for AI agents working in this repository.

## Two modes, two cores

Clip2Pod has two modes behind a tab bar:

- **NARRATE** — text → Edge TTS → MP3. Logic in `src-tauri/crates/clip2pod-core/`
  (AGPL-3.0-only).
- **RIP** — video URL → `yt-dlp` → MP3. Logic in `src-tauri/crates/ytdlfeed-core/`
  (GPL-3.0-only), vendored from the archived `FlocktimusPrime/yt-dlFeed` repo via
  `git subtree` on the `merge-ytdlfeed` branch (its history is in the log).

The shell (`src-tauri/src/`) wires both: one capture listener (`:4737`, routes by
`ytdlfeed_core::ytdlp::rippable_host` or the extension's `override_hint`), one feed
server (`:4738`, `/tts/*` + `/video/*`), two independent workers/queues. Rip config
and log live in `<config dir>/Clip2Pod2/rip/`, separate from the narrate ones.

**Licensing:** the combined binary links a GPL-3.0 crate into an AGPL-3.0 program.
AGPLv3 §13 permits this; the effective licence of the whole is AGPL-3.0. Keep
`ytdlfeed-core` marked GPL-3.0-only.

Event names and IPC commands: narrate keeps its bare names; rip is prefixed
(`rip:*` events, `rip_*` commands). Same for TS types (`RipJob`, `RipConfigView`, …).

## Agent skills

### Issue tracker

Issues and specs are tracked as GitHub issues in `FlocktimusPrime/Clip2Pod` (via the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Domain docs

Single-context: one `CONTEXT.md` and `docs/adr/` at the repo root (created lazily by `/domain-modeling`). See `docs/agents/domain.md`.
