# Clip2Pod — LAN Podcast Feed (RSS)

## Purpose

Serve the generated MP3s as a subscribable podcast feed on the local network,
so a phone podcast app on the same LAN can subscribe once and receive every
rendered article as an episode. No cloud, no accounts: the desktop app is the
feed host, and close-to-tray already keeps it resident.

## Decisions (settled during brainstorming)

- **Consumption:** phone podcast player on the same LAN. Feed is served over
  HTTP by the desktop app; not reachable from the internet.
- **Episode source:** all `*.mp3` files in the configured `output_dir`,
  scanned at request time. Survives reinstall, includes manually added files,
  no state to keep in sync.
- **Server shape:** a second, separate `tiny_http` server. The existing
  capture listener stays loopback-only on `127.0.0.1:4737`; the feed server
  binds `0.0.0.0:4738` so LAN devices can reach it. LAN peers can only ever
  GET audio/feed/cover — never POST captures.
- **Artwork:** bundled default cover only — one 1400×1400 PNG committed to
  the repo and embedded in the binary. No user-configurable artwork.
- **Episode metadata:** description snippet, source URL, duration, and author
  are all carried in the feed. The first three require writing extra ID3
  frames at generation time; author maps from the existing artist tag.

## Architecture

Three layers, matching the existing split:

1. **`clip2pod-core/src/feed.rs`** (new) — pure, unit-testable feed logic:
   scan a directory into `Episode` structs, build RSS 2.0 XML from them.
2. **`src-tauri/src/feed.rs`** (new) — thin HTTP shell: background thread,
   routes, path-traversal guard. Mirrors the `capture.rs` pattern.
3. **Pipeline + UI changes** — plumb source URL and summary through enqueue
   into the tagger; add a "Feed" button in the header that reveals the
   subscribe URL.

### Core: `clip2pod-core/src/feed.rs`

```rust
pub struct Episode {
    pub title: String,        // ID3 title, fallback: filename stem
    pub filename: String,     // bare filename, no path
    pub size: u64,            // bytes, for enclosure length
    pub modified: SystemTime, // pubDate
    pub artist: Option<String>,      // ID3 artist → itunes:author
    pub summary: Option<String>,     // COMM frame, description "summary"
    pub source_url: Option<String>,  // WOAF frame → <link>
    pub duration_ms: Option<u32>,    // TLEN frame → itunes:duration
}

pub fn scan_episodes(dir: &Path) -> Vec<Episode>;
pub fn build_rss(episodes: &[Episode], base_url: &str) -> String;
```

- `scan_episodes`: non-recursive scan for `*.mp3`, read ID3 frames
  best-effort (unreadable tag ⇒ filename-stem title, other fields `None`),
  sort newest-first by `modified`. Missing/unreadable dir ⇒ empty vec.
- `build_rss`: RSS 2.0 with the iTunes namespace.
  - Channel: title `Clip2Pod`, description
    `Articles narrated by Clip2Pod`, `<itunes:image href="{base_url}/cover.png"/>`,
    `<link>{base_url}/feed.xml</link>`, `<language>en</language>`.
  - Item per episode: `<title>` (XML-escaped), `<enclosure
    url="{base_url}/audio/{percent-encoded filename}" length="{size}"
    type="audio/mpeg"/>`, `<guid isPermaLink="false">{filename}</guid>`,
    `<pubDate>` RFC 2822 from `modified`.
  - Optional per-item elements, omitted entirely when the field is `None`:
    `<description>` (escaped summary), `<link>` (source URL),
    `<itunes:author>`, `<itunes:duration>` formatted `HH:MM:SS`.
- `base_url` is passed in by the caller (no trailing slash), e.g.
  `http://192.168.1.20:4738`.

### Core: `tagging.rs` changes

New signature (all call sites updated; best-effort semantics unchanged):

```rust
pub fn tag_mp3(
    path: &Path,
    title: &str,
    artist: &str,
    voice: &str,
    summary: &str,                // first ~300 chars of cleaned text
    source_url: Option<&str>,     // present only for fetched/captured articles
    duration_ms: Option<u32>,     // measured after render
) -> Result<(), id3::Error>
```

- `summary` → second `COMM` frame, `description: "summary"` (the existing
  voice comment keeps `description: "voice"`).
- `source_url` → `WOAF` frame (Official audio file webpage).
- `duration_ms` → `TLEN` frame (milliseconds, as ID3 defines it).
- Duration is measured by the queue worker after TTS render using the new
  core dependency `mp3-duration` (frame-accurate, VBR-safe). Measurement
  failure ⇒ `None` ⇒ no TLEN frame; never blocks tagging.

### Shell: `src-tauri/src/feed.rs`

Background thread spawned from `.setup()` in `lib.rs`, next to the capture
server. `tiny_http` bound to `0.0.0.0:4738` (port constant; bind failure is
logged and the app continues, same policy as capture).

Routes (GET only; anything else 404/405):

- `GET /feed.xml` — resolve `output_dir` from config (unset ⇒ empty feed),
  `scan_episodes` + `build_rss`. Base URL for enclosures is derived from the
  request's `Host` header (`http://{host}`), so whatever address the phone
  used automatically works in enclosure URLs. Missing Host header ⇒ fall back
  to detected LAN IP. Content-Type `application/rss+xml; charset=utf-8`.
- `GET /audio/<name>` — percent-decode `<name>`; reject any decoded name
  containing `/`, `\`, or `..` (400); serve the file from `output_dir` with
  Content-Type `audio/mpeg`; missing file ⇒ 404.
- `GET /cover.png` — bundled bytes via `include_bytes!`, Content-Type
  `image/png`. The 1400×1400 cover PNG is generated once with ImageMagick
  and committed to the repo (e.g. `src-tauri/assets/cover.png`).

### Pipeline: source URL + summary plumbing

- **Frontend state** (`stores.svelte.ts` / `+page.svelte`): new
  `sourceUrl: string | null`.
  - Set by Intake fetch (`extract` command result) and by the
    `article-captured` event (both know the origin URL).
  - Cleared whenever the script text is replaced wholesale (paste, clear/new).
    Manual edits to fetched text keep it.
- **Enqueue** (`api.ts` → `commands.rs`): enqueue call gains
  `source_url: Option<String>`. The frontend does not compute the summary —
  the queue worker derives it from the job text (first ~300 chars,
  whitespace-collapsed).
- **Queue** (`queue.rs`): job struct gains `source_url`; after render the
  worker measures duration, derives the summary from the job text, and passes
  all three new arguments to `tag_mp3`.

### UI: feed URL discovery

- New Tauri command `feed_url() -> String`: detect the LAN IP via the
  UDP-connect trick (`UdpSocket::bind("0.0.0.0:0")` +
  `connect("8.8.8.8:80")` + `local_addr()` — no packets sent, no new crate),
  return `http://{ip}:4738/feed.xml`. Detection failure ⇒
  `http://127.0.0.1:4738/feed.xml` (still useful for desktop testing).
- Header gains a "Feed" button opening a small modal (existing `Modal.svelte`)
  showing the URL with a copy button and a one-line hint
  ("Subscribe in your podcast app — same Wi-Fi, app must be running").

## Error handling

| Failure | Behavior |
| --- | --- |
| Port 4738 already bound | Log, continue; feed unavailable, app unaffected |
| `output_dir` unset or missing | Valid feed with zero items |
| Unreadable ID3 tag on a file | Episode still listed; title from filename, optional fields omitted |
| File deleted between feed fetch and download | 404; player skips |
| Duration measurement fails | No TLEN frame; feed omits `itunes:duration` |
| Traversal attempt in `/audio/` name | 400, nothing served |

## Out of scope (YAGNI)

- Configurable port, feed on/off toggle, user artwork, per-episode artwork,
  HTTPS/auth, internet-reachable hosting, retroactive tagging of old files
  (they appear with whatever tags they have).

## Verification

1. `cargo test -p clip2pod-core` (new tests: escaping, ordering, empty dir,
   optional-element omission, traversal-name rejection helper),
   `cargo check`, `npm run check` — all green.
2. `curl http://127.0.0.1:4738/feed.xml | xmllint --noout -` — valid XML;
   items match output folder newest-first.
3. `curl -sI http://127.0.0.1:4738/audio/<file>.mp3` — 200, `audio/mpeg`;
   `curl -s http://127.0.0.1:4738/audio/..%2fCargo.toml` — 400.
4. `curl -sI http://127.0.0.1:4738/cover.png` — 200, `image/png`.
5. Generate one episode from a fetched article → new feed item carries
   description, link, author, duration.
6. Feed modal shows LAN URL; acceptance: phone podcast app on same Wi-Fi
   subscribes and plays an episode.
