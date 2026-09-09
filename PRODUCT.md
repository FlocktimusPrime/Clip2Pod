# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary user: Keith — someone who consumes long-form written content by ear
(commute, chores, walks) and wants a personal audio library instead of a
read-it-later pile. Situation: they have just copied an article, hit a paywalled
page, or have a URL they want narrated, and they want the result to show up in
their phone's podcast app like any other show.

The job: intake text → clean it so it reads well aloud → hand it to a voice →
collect the finished MP3 in a private podcast feed on the home network.

Distributed publicly on GitHub under AGPL-3.0 for anyone with the same need, but
built and maintained around one person's workflow — not a managed product with a
user base, onboarding funnel, or support obligation.

## Product Purpose

Clip2Pod converts written content into narrated MP3 episodes and serves them as a
private podcast feed on the local network.

Input arrives four ways: clipboard paste, article-URL fetch (readability
extraction), a global hotkey from any app, and a companion browser extension that
captures the current page from the logged-in browser session (so paywalled
articles work). The text is cleaned for text-to-speech, narrated with one of
Microsoft Edge's neural voices, ID3-tagged, given a collision-safe filename, and
listed in a local RSS feed (port 4738) that a phone subscribes to over Wi-Fi.

Success: a copied article becomes a correctly tagged, correctly named episode in
the listener's podcast app with no manual file handling, and a backlog of dozens
of episodes doesn't all sound like the same narrator.

## Positioning

Clip2Pod turns *arbitrary* copied or paywalled text — not only RSS-published
articles — into a private podcast feed, using Microsoft Edge's free neural TTS and
running entirely on the user's own machine and LAN. No account, no subscription,
no cloud library, no per-character API bill.

Two mechanisms a generic "listen to articles" or TTS app doesn't combine:

- The browser extension sends the page as rendered in the user's logged-in
  session, so paywalled and subscriber-only content narrates like anything else.
- Voice selection deliberately rotates (round-robin within a gender pool;
  alternating gender when the author's is unknown; cycling state persisted across
  restarts) so a large generated backlog stays listenable.

## Operating Context

- Desktop app for **Windows and Linux** (Tauri 2 shell). Runs in the system tray;
  closing the window keeps it running. Global hotkey default `Ctrl+Alt+G`.
- Local servers: capture endpoint on `127.0.0.1:4737` (the browser extension
  POSTs here), RSS feed on `0.0.0.0:4738` (the phone reaches it over LAN).
- Listener loop: generate episodes → open the Feed view → scan the QR code or
  enter `http://<ip>:4738/feed.xml` in any podcast client on the same network →
  podcatcher downloads the episodes → **Delete episodes** clears the output folder
  for the next batch (confirmation shows the file count; never removes a file
  still rendering).
- Editing loop: paste or fetch text → **Find Junk** walks lines that read badly
  aloud (delete each with `Ctrl+D`), or **Clean for TTS** does it in one pass →
  review the auto-filled Title / Author / Filename title / Author gender →
  **Generate MP3** (queued, never blocks the UI).
- Keyboard-driven: every primary action has a visible shortcut; the app can be
  summoned and fed from anywhere via the global hotkey.

## Capabilities and Constraints

**Capabilities**

- Intake: clipboard, URL (`reqwest` + `dom_smoothie` readability), global hotkey,
  browser extension (Chrome + Firefox, Manifest V3).
- Text cleaning: smart punctuation → ASCII, emoji/pictograph strip,
  decorative-symbol removal, symbol-run strip, whitespace collapse. Autofill
  heuristics: title = line 1, author = shorter of lines 2/3, filename title =
  line 1.
- Junk-phrase finder: 17 default phrases, case-insensitive substring match with
  `*` wildcards, no auto-wrap; custom list persisted only when it differs from
  the defaults; highlight-to-add (`Ctrl+K`) and a boilerplate-scan suggester
  (`Ctrl+Shift+K`).
- Voice library: English non-cartoon Edge voices; default-enabled = a curated
  10-voice `en-US` pool (Ava, Andrew, Emma, Brian, Aria, Christopher, Eric,
  Jenny, Michelle, Steffan); per-gender pools; Unknown author
  gender alternates gender each generation; round-robin within a pool; cycling
  state persisted. Voices can be disabled from the Log or Manage Voices.
- Spoken intro: each episode opens with "Title. By Author." unless the text
  already starts that way.
- Background queue: single worker; jobs Queued / Processing / Done / Failed /
  Cancelled; **Clear Pending** never touches the in-flight job; **Cancel** aborts
  the in-flight render at the next chunk boundary; the footer Queue bar shows
  chunk-progress %; status lamp shows Idle / Queued n / rendering.
- Output: ID3v2 tags (Title, Artist, Album = "Clip2Pod", Comment = narrating
  voice); Windows-safe filename sanitize on both OSes; optional `C2P_` prefix;
  `(2)`/`(3)` collision suffix checked against disk *and* the queue; local RSS
  feed with cover art.
- Light and dark themes; all settings persist between sessions.

**Constraints**

- **Local-only, no accounts.** No telemetry, no sign-in, no hosted sync, no cloud
  library. The single outbound call is Edge TTS synthesis (the same service as
  Edge's "Read aloud"). Future work must not add accounts, hosted storage, or a
  cloud UI.
- Edge TTS is an unofficial endpoint (`msedge-tts`). Long text is chunked on
  sentence boundaries to stay under the per-request limit and the MP3 frames
  concatenated. Text is XML-escaped before it enters SSML or the service silently
  returns no audio.
- Global hotkey may not register under Linux Wayland; the tray menu is the
  fallback.
- Config: JSON at the platform config dir (`Clip2Pod2/config.json`); log capped at
  200 entries; atomic writes; panic hook writes `crash.log`.

## Brand Commitments

- **Name: "Clip2Pod"** — binding. It is the only fixed identity element.
- Nothing else in the current interface is a commitment. The shipped app uses a
  broadcast "production desk" framing (status lamp, `INTAKE` / `TRANSPORT` /
  `JUNK` / `TOOLS` panel labels, a `QUEUE` status bar under the editor,
  studio-dark default theme, daylight light theme). The user
  has explicitly said future design work may replace all of it — treat the
  current look as evidence and anti-reference, not direction.
- License: AGPL-3.0-only; distributed or network-hosted derivatives must ship
  source under the same terms.

## Evidence on Hand

- `README.md` and `ABOUT.md` — full feature walkthrough and end-to-end flow.
- `docs/screenshots/main-window-{dark,light}.png` — the current UI.
- `docs/superpowers/specs/2026-07-03-clip2pod-v2-design.md` — approved v2 design
  decisions and architecture.
- `new_branding/` — app icons (`app-png`), tray icons (`tray-png`), feed cover
  art (`cover`), source SVGs.
- No user testimonials, install counts, benchmarks, reviews, or press exist.
  Future work must not fabricate them.

## Product Principles

1. **The unit of work is the episode.** A copied article should reach the
   listener's podcast app as a tagged, correctly named MP3 with zero manual file
   handling.
2. **Stay local.** The machine and the LAN are the whole system; voice synthesis
   is the only outbound dependency, and that boundary does not move.
3. **Never lose or overwrite the user's work** — not pasted text (an empty
   clipboard warns instead of clearing the editor), not existing files
   (collision-safe naming), not the rendering job (Clear Pending spares it).
4. **A big backlog must stay listenable.** Voice variety is a feature, not a
   nicety.
5. **Keyboard-first.** Every primary action is reachable and labeled with its
   shortcut; the app runs from the tray and a global hotkey.
