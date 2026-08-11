# About Clip2Pod

Turn any article into a narrated MP3 — and listen to it as your own private podcast feed.

Clip2Pod turns clipboard text — articles, blog posts, newsletters, anything you've copied — into clean MP3 audio files you can listen to later, like a personal podcast feed. Paste text in, strip out the junk that clutters a web page (ads, bylines, "read more" links), and generate narration using Microsoft Edge's text-to-speech voices. Files are tagged and named automatically, and everything is queued in the background so you can keep working while audio renders.

The core loop: **clipboard → script → voice → MP3.**

## Who it's for

Anyone who wants to consume written content by ear — commuters, people who read a lot of long-form web articles, or anyone converting text into a personal audio library. The workflow: intake text, clean it up, hand it to a voice, monitor the output queue.

---

## Features

### 1. Clipboard intake

- **Paste Clipboard** (`Ctrl+Shift+V`) replaces the entire editor with whatever text is currently on the clipboard.
- If the clipboard is empty or has no text, the app warns instead of silently clearing the editor.

### 2. Text cleaning for TTS

A one-click **Clean for TTS** (`Ctrl+L`) pass prepares raw, messy pasted text for narration:

- Converts "smart" typographic punctuation to plain ASCII (curly quotes → straight quotes, em/en dashes → hyphens, non-breaking spaces → regular spaces).
- Strips emoji and other pictographic symbols.
- Removes decorative bullets/arrows/checkmarks/stars (•, ▶, ✓, ★, etc.) and copyright/trademark symbols (©, ®, ™).
- Strips runs of two or more stray symbols that aren't letters, numbers, or normal sentence punctuation.
- Collapses repeated whitespace and drops blank lines.
- Uses the cleaned first line to auto-fill the **Filename title** field.
- After cleaning, auto-fills the **Title** metadata field from line 1, and the **Author** field by picking whichever of line 2 or line 3 is shorter (a heuristic for byline detection, e.g. "By Jane Doe").

### 3. Junk phrase finder

Web articles are full of boilerplate that reads badly aloud ("Getty Images", "min read", "Related Stories", raw URLs, etc.). Clip2Pod finds these lines so you can review and remove them before generating audio:

- **Find Junk** (`Ctrl+F`) jumps to the next line containing a junk phrase, selects the whole line, and highlights the matched term. It does **not** auto-wrap — pressing it again after reaching the end asks if you want to wrap back to the top and keep searching.
- When no more matches are found, it offers to run Clean for TTS automatically.
- **Delete Line** (`Ctrl+D`) removes whatever line the cursor is on — the fast way to clear a flagged line.
- **Edit Junk Phrases** (`Ctrl+J`) opens an editor for the phrase list: one phrase per line, case-insensitive substring match, with a "Restore Defaults" button. Custom lists persist between sessions; if you set it back to exactly the defaults, it's stored as "using defaults" rather than a redundant copy.
- Default junk phrases: `credit:`, `getty images`, `http`, `listen to article`, `min read`, `read more`, `read this article for free`, `related links`, `related stories`, `unsplash`, `view image in full size`, `view original`.

### 4. Metadata bar

Four fields above the script editor control the output file:

| Field | Purpose |
|---|---|
| **Title** | Written to the MP3's ID3 Title tag. Auto-filled from line 1 after cleaning; editable. |
| **Author** | Written to the MP3's ID3 Artist tag. Auto-filled from the likely byline line after cleaning; editable. |
| **Filename title** | Used to build the output filename. Auto-filled from the cleaned first line; editable. |
| **Author gender** | `Unknown` / `Male` / `Female` — controls which voice pool is used for narration (see below). |

### 5. Voice library

- On launch, Clip2Pod fetches the full list of English Edge TTS voices, excluding any tagged "cartoon" (novelty voices unsuited to narration).
- Out of the box, only standard `en-US` voices are enabled (the "Multilingual" variant of each voice is a duplicate and stays off by default to avoid doubling up).
- **Manage Voices** (`Ctrl+M`) opens a searchable table of every available voice (name, gender, language, country, locale, category) with checkboxes to enable/disable each one, plus **Enable All** / **Disable All** shortcuts.
- Selecting a voice and clicking **Preview Voice** generates and plays a short sample sentence in that voice, so you can audition it before enabling it for use.
- The sidebar shows a running count, e.g. "Voices enabled: 12/40 male, 15/38 female".

### 6. Voice selection & variety cycling

Clip2Pod deliberately avoids using the same voice every time, so a backlog of generated episodes doesn't sound monotonous:

- If **Author gender** is set to Male or Female, every generation for that piece uses a voice from that gender's enabled pool.
- If **Author gender** is Unknown, the app alternates gender on each generation (Male, then Female, then Male, ...), starting from whichever gender wasn't used last.
- Within a gender, voices are cycled round-robin through the enabled list, so repeated generations rotate through all enabled voices before repeating.
- Cycling position and "last gender used" persist across restarts, so variety continues seamlessly rather than resetting every launch.
- If a voice turns out to be flaky or mispronounces things badly, you can right-click its entry in the **Log** and choose "Disable this voice" to remove it from the rotation immediately, without opening Manage Voices.

### 7. Background generation queue

- **Generate MP3** (`Ctrl+Enter`) doesn't block the UI — it cleans the current text, picks a voice, and queues a job.
- A single background worker processes jobs one at a time, in order.
- A status lamp in the header reflects real-time state: `● IDLE`, `● QUEUED n`, or `● RENDERING` while actively encoding.
- **Open Queue** (`Ctrl+Q`) shows pending/in-progress jobs with status, title, output filename, voice, and timestamps.
- **Clear Pending** cancels every job that hasn't started yet (logged as "Cancelled") without interrupting whatever job is currently rendering.

### 8. Filename handling

- Filenames are sanitized to remove characters Windows won't allow, trimmed of trailing dots/spaces, and length-capped.
- An optional **"Prefix filenames with C2P"** checkbox prepends `C2P_` to every output filename.
- If the target filename already exists on disk, or already belongs to a job still queued/processing, Clip2Pod automatically appends `(2)`, `(3)`, etc. until it finds a free name — never silently overwriting a file.

### 9. ID3 tagging

Each generated MP3 gets ID3 tags written automatically (tagging is best-effort and skipped gracefully if unavailable):

- Title → Title metadata field
- Author → Artist metadata field
- Album is always set to "Clip2Pod"
- A Comment tag records which voice narrated the file

### 10. Generation log

- Every job — queued, completed, failed, or cancelled — is recorded with a timestamp, status, title, voice used, output filename, and details (e.g. an error message).
- **Open Log** (`Ctrl+Shift+L`) shows the full history (most recent 200 entries), searchable by title.
- Right-click any entry to disable the voice that narrated it, directly from the log.
- **Clear Log** wipes the history.

### 11. Settings persistence

All of the following are remembered between sessions: output directory, filename prefix preference, author-gender selection, voice enable/disable list, voice-cycling state, custom junk phrases, generation log, and the light/dark theme choice.

### 12. Light/dark theme

A rocker-style switch in the header toggles between a dark "night" theme and a light "day" theme. The choice is saved and restored on next launch.

### 13. Episode cleanup

Once your podcast app has downloaded the generated episodes, the **Delete episodes** button (sidebar, Output section) clears the output folder in one click:

- Deletes every `.mp3` the podcast feed currently lists — exactly the set your podcatcher sees.
- Asks for confirmation first, showing how many files will be removed (the output folder may contain more than you think).
- Never deletes the output file of a job that is still queued or rendering.
- If the folder has no episodes, it says so instead of showing an empty confirmation.

---

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+V` | Paste Clipboard (overwrites editor) |
| `Ctrl+D` | Delete current line |
| `Ctrl+F` | Find next junk phrase |
| `Ctrl+L` | Clean text for TTS |
| `Ctrl+J` | Edit junk phrases |
| `Ctrl+M` | Manage voices |
| `Ctrl+Q` | Open generation queue |
| `Ctrl+Shift+L` | Open generation log |
| `Ctrl+Enter` | Generate MP3 |

---

## End-to-end flow

1. Copy an article or block of text somewhere else.
2. **Paste Clipboard** into Clip2Pod.
3. Run **Find Junk** repeatedly (deleting flagged lines) or just run **Clean for TTS** directly — either way, boilerplate and stray symbols get stripped.
4. Review/edit the auto-filled Title, Author, Filename title, and Author gender.
5. Press **Generate MP3**. The job is queued, a voice is picked (respecting gender preference and rotation), and the app keeps working while it renders.
6. The finished MP3 lands in your chosen output folder, ID3-tagged, with a collision-safe filename — and a permanent record in the Log.
7. After your podcatcher refreshes the feed and downloads the episodes, **Delete episodes** clears the output folder for the next batch.

## Replicating this app

The only external dependency that matters functionally is **Microsoft Edge's text-to-speech service** for voice synthesis and voice listing (voice metadata, gender, locale, and the actual audio rendering all come from it). Everything else described above — the cleaning pipeline, junk-phrase matching, voice cycling logic, queue, filename collision handling, and ID3 tagging — is plain application logic that can be reimplemented in any language or UI framework on top of any TTS engine that can (a) list available voices with a gender/locale, and (b) synthesize text to an audio file.
