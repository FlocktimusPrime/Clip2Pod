# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary user: Keith — someone who consumes long-form content by ear (commute,
chores, walks) and wants a personal audio library instead of a read-it-later pile
*and* a stack of talks/interviews they'll never sit down to watch. Situation: they
have just copied an article, hit a paywalled page, or have a video URL, and they
want the result to show up in their phone's podcast app like any other show.

The job: send something in → let it become a clean, tagged MP3 → collect it in a
private podcast feed on the home network.

Distributed publicly on GitHub under AGPL-3.0 for anyone with the same need, but
built and maintained around one person's workflow — not a managed product with a
user base, onboarding funnel, or support obligation.

## Product Purpose

Clip2Pod produces narrated or ripped MP3 episodes and serves them as private
podcast feeds on the local network. It has two modes behind a header tab:

- **Narrate** — text (clipboard paste, article-URL fetch with readability
  extraction, a global hotkey, or a browser extension that captures the current
  page from the logged-in session so paywalled articles work) is cleaned for TTS,
  narrated with a Microsoft Edge neural voice, ID3-tagged, given a collision-safe
  filename, and listed in `/tts/feed.xml`.
- **Rip** — a video URL (pasted, or sent from the extension) is handed to `yt-dlp`,
  which downloads the audio track as an MP3 (embed thumbnail/metadata/chapters,
  SponsorBlock, volume boost), duration-tagged and listed in `/video/feed.xml`.

Both feeds are served from one LAN RSS server (port 4738); the browser extension
posts to one capture listener (port 4737) that routes by video-host match or an
explicit right-click override.

Success: a copied article or a video link becomes a correctly tagged, correctly
named episode in the listener's podcast app with no manual file handling; a
backlog of dozens of narrated episodes doesn't all sound like the same narrator;
and narrated blog posts don't get shuffled in with two-hour talks (the two feeds
stay separate).

## Positioning

Clip2Pod turns *arbitrary* copied or paywalled text — not only RSS-published
articles — and *any* `yt-dlp`-supported video into private podcast feeds, using
Microsoft Edge's free neural TTS and the system `yt-dlp`, running entirely on the
user's own machine and LAN. No account, no subscription, no cloud library, no
per-character API bill.

Mechanisms a generic "listen to articles" or TTS app doesn't combine:

- The browser extension sends the page as rendered in the user's logged-in
  session, so paywalled and subscriber-only content narrates like anything else —
  and the same button reaches the ripping path for video pages.
- Voice selection deliberately rotates (round-robin within a gender pool;
  alternating gender when the author's is unknown; state persisted) so a large
  narrated backlog stays listenable.
- Narrated and ripped audio are kept as two feeds, because a 6-minute blog post
  and a 2-hour conference talk are different listening experiences.

## Operating Context

- Desktop app for **Windows and Linux** (Tauri 2 shell). Runs in the system tray;
  closing the window keeps it running; left-click the tray icon to show it. Global
  hotkey default `Ctrl+Alt+G` (narrate intake).
- Local servers: capture endpoint on `127.0.0.1:4737` (the extension POSTs
  `{ url, html, override_hint? }`), RSS feed on `0.0.0.0:4738` serving
  `/tts/feed.xml` + `/video/feed.xml` (the phone reaches it over LAN).
- **Rip depends on `yt-dlp` and `ffmpeg` on the PATH** — not bundled. A startup
  probe surfaces an install notice on the Rip tab if either is missing; the
  `yt-dlp` binary path is configurable.
- Listener loop: make episodes → open **Feed** on the tab you want → scan the QR
  or enter the URL in any podcast client on the same network → podcatcher
  downloads → **Delete all** clears that mode's folder (confirmation shows the
  count; never removes a file still being written).
- Narrate editing loop: paste or fetch → **Find Junk** / **Clean for TTS** →
  review Title / Author / Filename title / Author gender → **Generate MP3**
  (queued).
- Rip loop: paste a URL → **Download** → watch the queue.
- The two workers run concurrently; one header lamp reads both.

## Capabilities and Constraints

**Capabilities**

- **Narrate intake:** clipboard, URL (`reqwest` + `dom_smoothie` readability),
  global hotkey, browser extension.
- **Narrate cleaning:** smart punctuation → ASCII, emoji/pictograph strip,
  decorative-symbol removal, symbol-run strip, whitespace collapse. Autofill:
  title = line 1, author = shorter of lines 2/3, filename title = line 1.
- **Junk-phrase finder:** 17 default phrases, case-insensitive substring match
  with `*` wildcards, no auto-wrap; highlight-to-add; boilerplate-scan suggester.
- **Voice library:** English non-cartoon Edge voices; default-enabled = a curated
  10-voice `en-US` pool; per-gender pools; Unknown author gender alternates each
  generation; round-robin within a pool; cycling state persisted; disable a voice
  from the Log.
- **Recognized authors:** gender remembered per author name (exact match on the
  Author field, first set the moment Male/Female is picked); pre-filled and
  visually flagged on a future article from the same byline; managed (rename,
  edit, delete, merge byline variants) via Manage Author Genders.
- **Spoken intro:** "Title. By Author." unless the text already starts that way.
- **Rip:** `yt-dlp` with an editable args template (app appends `-P <dir>` and the
  URL); `--newline` progress parsing (percent / speed / stage); **Stop** kills the
  process; post-download duration (`TLEN`) tagging; configurable `yt-dlp` binary;
  a startup doctor probe for `yt-dlp` + `ffmpeg`.
- **Video-host routing:** a static allowlist (`RIPPABLE_HOSTS`) matched offline,
  overridable per-capture from the extension's right-click menu.
- **Queues:** two independent single-worker queues, running concurrently; jobs
  Queued / Processing / Done / Failed / Cancelled; **Clear Pending** never touches
  the in-flight job.
- **Output:** ID3 tags; Windows-safe filename sanitize on both OSes; optional
  `C2P_` prefix (narrate); `(2)`/`(3)` collision suffix checked against disk *and*
  queue; two local RSS feeds with cover art.
- Light and dark themes; sectioned Settings (General / Narrate / Rip); all
  settings persist.

**Constraints**

- **Local-only, no accounts.** No telemetry, no sign-in, no hosted sync, no cloud
  library. Outbound calls: Edge TTS synthesis (narrate) and whatever site `yt-dlp`
  fetches from (rip). Future work must not add accounts, hosted storage, or a
  cloud UI.
- Edge TTS is an unofficial endpoint (`msedge-tts`). Long text is chunked on
  sentence boundaries and MP3 frames concatenated. Text is XML-escaped before SSML
  or the service silently returns no audio.
- Rip is not self-contained: `yt-dlp` + `ffmpeg` must be installed by the user.
  `yt-dlp` needs frequent updating as sites change.
- Global hotkey may not register under Linux Wayland; the tray menu is the
  fallback.
- Config: JSON at the platform config dir — narrate at `Clip2Pod2/config.json`,
  recognized authors at `Clip2Pod2/authors.json`, rip at
  `Clip2Pod2/rip/config.json`. Logs capped at 200 entries each; atomic writes;
  panic hook writes `crash.log`.
- **Licensing:** `clip2pod-core` is AGPL-3.0-only, `ytdlfeed-core` is GPL-3.0-only
  (vendored from the archived yt-dlFeed repo). AGPLv3 §13 permits the combination;
  the whole binary is effectively AGPL-3.0.

## Brand Commitments

- **Name: "Clip2Pod"** — binding. The only fixed identity element. It absorbed the
  sibling app "yt-dlFeed" as the RIP tab; that name is retired.
- Nothing else in the current interface is a commitment. The shipped app uses a
  broadcast "production desk" framing (status lamp, mono/uppercase chrome,
  `INTAKE` / `TRANSPORT` / `JUNK` / `TOOLS` sidebar labels on the NARRATE tab, a
  header mode toggle, studio-dark default theme). Future design work may replace
  all of it — treat the current look as evidence and anti-reference, not
  direction.
- License: AGPL-3.0-only; distributed or network-hosted derivatives must ship
  source under the same terms.

## Evidence on Hand

- `README.md` and `ABOUT.md` — full feature walkthrough and end-to-end flows.
- `DESIGN.md` — the "Production Desk" design system, updated for the two-tab shell.
- `docs/screenshots/main-window-{dark,light}.png` — v0.8.3 (Feed icon):
  NARRATE in dark theme, RIP in light theme.
- `docs/design-reconciliation.md` — history of the yt-dlFeed merge (resolved).
- `docs/superpowers/specs/` — v2 design record, RSS feed design, Firefox
  extension workflow.
- `branding/feed/` — current Feed mark (paragraph lines bending into the RSS
  arcs): source SVGs, exported icons, tray glyphs, per-feed covers, web icons,
  generator (`src/`), usage rules in `GUIDELINES.md`. Retired marks (red mic,
  document-and-waves, C2P) are in `branding/archive/`.
- No user testimonials, install counts, benchmarks, reviews, or press exist.
  Future work must not fabricate them.

## Product Principles

1. **The unit of work is the episode.** A copied article or a video link should
   reach the listener's podcast app as a tagged, correctly named MP3 with zero
   manual file handling.
2. **Stay local.** The machine and the LAN are the whole system; TTS synthesis and
   `yt-dlp`'s fetches are the only outbound traffic, and that boundary does not
   move toward accounts or hosted storage.
3. **Never lose or overwrite the user's work** — not pasted text (an empty
   clipboard warns), not existing files (collision-safe naming), not the running
   job (Clear Pending spares it).
4. **A big backlog must stay listenable.** Voice variety is a feature; narrated
   and ripped audio stay in separate feeds.
5. **Keyboard-first (on NARRATE).** Every primary narrate action is reachable and
   labeled with its shortcut; the app runs from the tray and a global hotkey.
6. **Don't pretend to own dependencies you don't.** `yt-dlp` isn't bundled; the
   app detects it, explains how to install it, and points at update instructions
   when a rip fails.
