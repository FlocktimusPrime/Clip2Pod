# About Clip2Pod

Turn an article — or a video's audio — into your own private podcast feed.

Clip2Pod produces MP3 episodes and serves them over a local RSS feed on your home
network, so your phone's podcast app can subscribe and download them like any
other show. It has two modes, chosen with a tab in the header:

- **Narrate** — clipboard text or a web article → cleaned script → an MP3 read
  aloud by a Microsoft Edge neural voice.
- **Rip** — a video URL → the audio track downloaded as an MP3 by `yt-dlp`.

Each mode has its own episode folder and its own feed. Everything is queued in the
background so you can keep working (or rip a video while an article narrates).

## Who it's for

Anyone who consumes long-form content by ear — commuters, walkers, people with a
big read-it-later pile or a stack of talks and interviews they'll never sit down
to watch. The workflow: send something in, let it process, subscribe once on your
phone, and new episodes just show up.

---

## NARRATE mode

### 1. Four ways in

- **Paste Clipboard** (`Ctrl+Shift+V`) replaces the editor with whatever text is
  on the clipboard. An empty or non-text clipboard warns instead of clearing the
  editor.
- **Fetch by URL** — type an article URL in the Intake field; readability
  extraction pulls out title, author, and body.
- **Global hotkey** (`Ctrl+Alt+G`) — summons the window from any app and pastes
  the clipboard, in one keystroke.
- **Browser extension** — click it (or right-click → "Narrate this page") to send
  the page as rendered in your logged-in session, so paywalled and
  subscriber-only articles work.

### 2. Text cleaning for TTS

One-click **Clean for TTS** (`Ctrl+L`):

- Converts smart punctuation to plain ASCII (curly quotes, em/en dashes,
  non-breaking spaces).
- Strips emoji, pictographs, decorative bullets/arrows/stars, and ©/®/™.
- Removes runs of two or more stray symbols.
- Collapses repeated whitespace, drops blank lines.
- Auto-fills **Title** from line 1, **Author** from the shorter of lines 2/3 (a
  byline heuristic), and **Filename title** from line 1. For extracted articles,
  readability already found the real title/author, so cleaning only fills gaps.

### 3. Junk phrase finder

Boilerplate that reads badly aloud ("Getty Images", "min read", "Related
Stories", raw URLs):

- **Find Junk** (`Ctrl+F`) jumps to the next line containing a junk phrase,
  selects it, and highlights the match. No auto-wrap — it asks before wrapping to
  the top.
- **Add Junk Phrase** (`Ctrl+K`) adds the current selection to the list.
- **Suggest Junk** (`Ctrl+Shift+K`) scans for page furniture (bylines, "N min
  read", share rows, dates, all-caps banners) and walks you through each.
- **Delete Line** (`Ctrl+D`) removes the cursor's line.
- **Edit Junk Phrases** (`Ctrl+J`) — one phrase per line, case-insensitive
  substring match, `*` wildcard, "Restore Defaults". A custom list persists only
  when it differs from the defaults.
- Defaults: `credit:`, `getty images`, `http`, `image by author`, `image
  credit`, `listen to article`, `member-only`, `min read`, `photo from`, `read
  more`, `read it free`, `read this article for free`, `related links`, `related
  stories`, `unsplash`, `view image in full size`, `view original`.

### 4. Metadata bar

| Field | Purpose |
|---|---|
| **Title** | ID3 Title tag. Auto-filled from line 1; editable. |
| **Author** | ID3 Artist tag. Auto-filled from the likely byline; editable. |
| **Filename title** | Builds the output filename. Auto-filled from line 1. |
| **Author gender** | `Unknown` / `Male` / `Female` — picks the voice pool. Pre-filled and ringed when the Author field exactly matches someone in **Manage Author Genders** (see below). |

### 5. Voice library

- On launch, Clip2Pod fetches every English Edge TTS voice, minus "cartoon"
  novelty voices.
- Default-enabled: a curated ten-voice `en-US` pool (Ava, Andrew, Emma, Brian,
  Aria, Christopher, Eric, Jenny, Michelle, Steffan).
- **Manage Voices** (`Ctrl+M`) — a searchable table (name, gender, language,
  country, locale, category) with per-voice checkboxes, Enable/Disable All, and
  **Preview Voice** (plays a sample sentence).
- The Settings dialog and the narrate sidebar show a running enabled count.

### 6. Voice selection & variety cycling

- Author gender Male/Female → every generation uses that pool.
- Author gender Unknown → the app alternates gender each generation, starting from
  whichever wasn't used last.
- Within a gender, voices rotate round-robin through the enabled list.
- Cycling position and "last gender" persist across restarts.
- Right-click a bad take in the **Log** → "Disable this voice" removes it from the
  rotation immediately.

### 7. Recognized authors

- The first time you pick **Male** or **Female** for an author, Clip2Pod
  remembers that gender against the exact text in the **Author** field —
  picking **Unknown** never creates an entry, since there's nothing to
  remember yet.
- The next article whose Author field exactly matches a remembered name
  pre-fills the gender picker as soon as it loads, and rings the selected
  cell so you can tell it came from memory rather than being left over from
  the previous article. The check re-runs live if you edit the Author field.
- Picking a different gender for a recognized author's article overwrites
  their saved value — including picking Unknown again, which un-recognizes
  them until you pick Male/Female for them again.
- **Manage Author Genders** (`Ctrl+G`, or Settings → Narrate) lists everyone
  remembered: search, rename (fix a byline typo), edit gender, delete, and
  merge two rows into one (fold a byline variant like "J. Doe" into "Jane
  Doe" — the surviving row's name and gender are kept as-is).

### 8. Spoken intro

Each episode opens with "*Title. By Author.*" — skipped automatically if the text
already starts that way.

### 9. Output

- ID3v2 tags: Title, Artist, Album = "Clip2Pod", a Comment recording the voice.
- Filenames sanitized for Windows on every OS, trimmed, length-capped.
- Optional **Prefix filenames with C2P** checkbox.
- On a name collision (on disk *or* in the queue), `(2)` / `(3)` is appended —
  never a silent overwrite.

---

## RIP mode

### 1. Intake

- Paste a video URL and hit **Download**, or **Paste link** to take it from the
  clipboard and start immediately.
- Or send it from the browser extension — a known video host (YouTube, Vimeo,
  SoundCloud, Twitch, …) auto-routes here; right-click → "Rip this page's audio"
  forces it for anything else.

### 2. Download

`yt-dlp` runs with an editable args template (Settings → Rip). The default:
extract audio to MP3, embed thumbnail / metadata / chapters, run SponsorBlock,
apply a `volume=1.5` boost, restrict filenames. The app always appends the output
folder and the URL itself — those can't drift.

- Live per-job progress (percent, speed, postprocessor stage) parsed from
  `yt-dlp --newline`.
- **Stop** kills the current `yt-dlp` process; the job logs as "Cancelled", no
  partial file left behind.
- After each job, Clip2Pod measures the MP3 duration and writes the `TLEN` frame
  (`yt-dlp` doesn't), so the feed carries lengths.

### 3. Requirements & the doctor

Rip needs **`yt-dlp` and `ffmpeg` on your PATH** — Clip2Pod bundles neither
(`yt-dlp` changes too often to ship). On startup and after a Settings save it
probes for both:

- Missing → an inline notice at the top of the Rip tab with the `winget install`
  commands and a **Re-check** button.
- Present → Settings → Rip shows the detected `yt-dlp` version.
- A failed rip also reminds you that a stale `yt-dlp` is the usual cause
  (`yt-dlp -U`).
- **yt-dlp binary** field in Settings → point at a specific build (e.g. a
  nightly) without touching your PATH.

### 4. Episodes table

Title, channel, length, size, date added, per-row **Delete**. **Refresh**
re-scans the folder; **Delete all** clears every MP3 the feed lists (with an OS
confirmation dialog).

---

## Shared

### Background queues

- Two independent workers — narrate (an Edge TTS websocket) and rip (a `yt-dlp`
  subprocess) — run concurrently.
- One header **status lamp** reads both: `IDLE`, `QUEUED n` (combined),
  `RENDERING` (narrate busy), `RIPPING` (rip busy), `ON AIR` (both).
- Each tab has its own Queue section: the job in progress with a progress bar,
  jobs waiting, failures with an expandable reason, the last success.
- **Clear pending** cancels queued jobs without touching the in-flight one.

### Two podcast feeds

- A local RSS server on port `4738` serves `/tts/feed.xml` (narrated articles) and
  `/video/feed.xml` (ripped audio), each with cover art. There is no combined
  feed — subscribe to whichever you want.
- **Feed** in the header opens a dialog for the *active* tab's feed: the URL, a QR
  code to scan from your phone, a Copy button, and firewall help.
- Firewall help gives you the exact command to open the inbound port (per OS) plus
  the common gotchas — it does not try to change the firewall itself.

### Logs

- Each mode keeps its own log (most recent 200 entries), searchable by title. The
  header **LOG** button opens the active tab's.
- Narrate's log records the voice used and lets you disable it by right-click.

### Settings

One dialog, three sections:

- **General** — theme (dark/light), start minimized to tray, launch at sign-in.
- **Narrate** — episode folder, C2P filename prefix, Manage voices, Manage
  Author Genders.
- **Rip** — episode folder, yt-dlp arguments (+ reset), yt-dlp binary path.

All settings persist between sessions.

### Tray & window

Closing the window keeps the app running in the system tray (so the capture
listener and feeds stay reachable). **Left-click** the tray icon to show the
window; right-click for Show / Hide / Quit.

---

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Alt+G` | **Global** — summon Clip2Pod and paste the clipboard, from any app |
| `Ctrl+Shift+V` | Paste Clipboard (NARRATE) |
| `Ctrl+D` | Delete current line (NARRATE) |
| `Ctrl+F` | Find next junk phrase |
| `Ctrl+K` | Add selection as junk phrase |
| `Ctrl+Shift+K` | Suggest junk phrases |
| `Ctrl+L` | Clean text for TTS |
| `Ctrl+J` | Edit junk phrases |
| `Ctrl+M` | Manage voices |
| `Ctrl+G` | Manage Author Genders |
| `Ctrl+Enter` | Generate MP3 |
| `Ctrl+Shift+L` | Open the log (the active tab's) |
| `Ctrl+,` | Open Settings |

---

## End-to-end flows

**Narrate:** copy an article → **Paste Clipboard** → **Find Junk** / **Clean for
TTS** → review the metadata → **Generate MP3** → it queues, a voice is picked,
the tagged file lands in the narrate folder and the `/tts` feed.

**Rip:** on the RIP tab, paste a video URL → **Download** → `yt-dlp` runs → the
tagged MP3 lands in the rip folder and the `/video` feed. (Or right-click the page
in your browser → "Rip this page's audio".)

**Listen:** open **Feed** on the tab you want → scan the QR in your phone's
podcast app → episodes appear as they finish → **Delete all** clears the folder
once your podcatcher has them.

## Replicating this app

Narrate's only functional external dependency is **Microsoft Edge's TTS service**
(voice list + synthesis); everything else — cleaning, junk matching, voice
cycling, the queue, collision handling, ID3 tagging — is plain logic on top of any
TTS engine that can list voices and synthesize to a file. Rip is a thin shell
around **`yt-dlp`**: arg-template building, `--newline` progress parsing, and
duration tagging. The RSS feed is a directory scan plus a hand-built RSS 2.0
string per mode.
