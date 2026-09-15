---
name: Clip2Pod
description: A one-person broadcast desk that turns copied text — or a video's audio — into a private podcast feed.
colors:
  bg: "#161826"
  panel: "#1d1f3d"
  panel-raised: "#262a60"
  line: "rgba(233, 233, 237, 0.16)"
  line-soft: "rgba(233, 233, 237, 0.10)"
  text: "#e9e9ed"
  muted: "#9b9bb8"
  signal-lavender: "#b5abfc"
  signal-lavender-dim: "#6f63b0"
  accent-ink: "#161826"
  danger: "#ef5b6a"
  ok: "#5fd0a0"
  idle: "#3a3d5c"
typography:
  ident:
    fontFamily: "ui-monospace, 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "0.22em"
  control:
    fontFamily: "ui-monospace, 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace"
    fontSize: "11px"
    fontWeight: 600
    lineHeight: 1
    letterSpacing: "0.1em"
  label:
    fontFamily: "ui-monospace, 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace"
    fontSize: "10px"
    fontWeight: 600
    lineHeight: 1
    letterSpacing: "0.14em"
  body:
    fontFamily: "system-ui, 'Segoe UI', Roboto, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  prose:
    fontFamily: "system-ui, 'Segoe UI', Roboto, sans-serif"
    fontSize: "13.5px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  field:
    fontFamily: "ui-monospace, 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "normal"
  data:
    fontFamily: "ui-monospace, 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace"
    fontSize: "12.5px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  note:
    fontFamily: "system-ui, 'Segoe UI', Roboto, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  kbd:
    fontFamily: "ui-monospace, 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace"
    fontSize: "9px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "normal"
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
  pill: "50%"
spacing:
  xs: "4px"
  sm: "5px"
  md: "8px"
  lg: "10px"
  band-y: "12px"
  xl: "16px"
components:
  button:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.text}"
    typography: "{typography.control}"
    rounded: "{rounded.sm}"
    padding: "7px 12px"
  button-hover:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.text}"
  button-primary:
    backgroundColor: "{colors.signal-lavender}"
    textColor: "{colors.accent-ink}"
    typography: "{typography.control}"
    rounded: "{rounded.sm}"
    padding: "7px 12px"
  button-small:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.text}"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    padding: "3px 8px"
  field:
    backgroundColor: "{colors.bg}"
    textColor: "{colors.text}"
    typography: "{typography.field}"
    rounded: "{rounded.sm}"
    padding: "6px 8px"
  segmented-cell:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    padding: "0 14px"
    height: "31px"
  segmented-cell-active:
    backgroundColor: "{colors.signal-lavender}"
    textColor: "{colors.accent-ink}"
  band:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.text}"
    padding: "12px 16px"
  modal:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.text}"
    rounded: "{rounded.lg}"
    padding: "12px 16px"
  coach-bar:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.md}"
    padding: "8px 14px"
  toast:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.text}"
    typography: "{typography.data}"
    rounded: "{rounded.sm}"
    padding: "9px 14px 9px 28px"
---

# Design System: Clip2Pod

## Overview

**Creative North Star: "The Production Desk"**

Clip2Pod is a one-person broadcast console. The window is a fixed, non-scrolling
desk — a header rail with the station ident, a mode toggle, and a status lamp,
then a stack of equipment bands that never grows past the viewport. Every part of
the frame is a piece of equipment: the lamp tells you whether the desk is live,
the transport buttons move a script through intake → clean → render, the rail
under the queue bar charges left-to-right as audio is cut. Nothing on the desk is
decorative; if an element is visible, it either reports state or performs an
action.

The desk does two jobs, chosen by a segmented toggle in the header. **NARRATE**
is the script desk: a metadata strip, a large editor with a control column beside
it, a queue rail pinned to the bottom. **RIP** is the same desk configured for
pulling a video's audio track: an intake band, a bounded queue, and an episode
table that fills the rest of the frame and scrolls inside its own borders. The
ident, the lamp, and the FEED / LOG / SETTINGS buttons are shared; only the
working surface below them changes. One console, two intake paths.

The palette is a dark studio by default — deep indigo panels stacked by
lightness, one soft lavender that means *signal* (the lamp when queued, the
primary action, the render rail) and one red that means *failure* (a job that
failed, a render in progress, a destructive confirm). Type does the rest of the
work: a monospace voice for every control, label, and data readout, uppercased
and letter-spaced like console silkscreen, with a plain system sans reserved for
the two places real prose lives — the article in the editor and the body copy
inside dialogs. The result is dense, calm, and legible at a glance, the way a
mixing desk is: you learn the surface once and then operate it without looking.

This is a keyboard-first instrument on the NARRATE side — every primary action
shows its shortcut on its own face (`Ctrl+L`, `Ctrl+Enter`), and the whole app
can be summoned and fed from a global hotkey. The desk favours a settled,
equipment-grade restraint over expression — motion is limited to state (a lamp
pulse, a rail easing toward the next chunk, a dialog rising 6px) and never to
ornament.

**Key Characteristics:**
- Both tabs are the same fixed desk: the window is exactly the viewport, `overflow: hidden`, and only inner regions (the editor, the sidebar, the queue, the episode table, a dialog body) ever scroll.
- One shared header — ident, `NARRATE` / `RIP` segmented toggle, one combined status lamp, `FEED` / `LOG` / `SETTINGS` — over a content area that shows one tab's desk at a time.
- Monospace, uppercase, letter-spaced type for all chrome; system sans only for authored prose.
- One accent (Signal Lavender) for *signal*, one red for *failure*, everything else neutral indigo.
- Tonal depth: surfaces separate by lightness step, not by shadow. Shadows mark only floating layers.
- A single circular-marker vocabulary — the lamp bulb, the toast dot, the queue-item disc, the doctor strip's dot — tinted to state.
- Every NARRATE control wears its keyboard shortcut.

## Colors

A dark indigo studio with a single cool accent; the light theme is the same room
with the lights on — near-white panels on a lilac-grey floor, the accent
darkened to hold contrast.

### Primary
- **Signal Lavender** (`#b5abfc` dark / `#5b4fc4` light): The one accent. Used for the *live signal* — the status lamp when a job is queued, the `Generate MP3` primary button, the render-progress rail, the active segmented cell, the editor caret and selection, focus outlines, sort arrows, `accent-color`. It is never used as a surface fill for passive content.
- **Signal Lavender Dim** (`#6f63b0` dark / `#b5abfc` light): The muted echo of the accent — button hover borders, scrollbar thumbs, the coach bar's border, the indeterminate progress stripe's darker band. Signals "interactive, not active".

### Neutral
- **Desk Black** (`#161826` dark / `#f2f1fa` light): The window ground and the editor/field background — the deepest layer, where authored text sits.
- **Panel** (`#1d1f3d` dark / `#ffffff` light): The equipment surface — every desk band, the sidebar, dialog body, table header. One step up from the ground.
- **Panel Raised** (`#262a60` dark / `#ffffff` light): The top tonal layer — button faces, the lamp housing, segmented cells, toasts, the coach bar, the doctor strip. In light theme it collapses to plain white and relies on borders.
- **Ink** (`#e9e9ed` dark / `#1f2140` light): Primary text.
- **Muted Ink** (`#9b9bb8` dark / `#5f6186` light): Labels, placeholders, secondary and "waiting" queue items, the doctor strip's install commands.
- **Accent Ink** (`#161826` dark / `#ffffff` light): Text that sits *on* Signal Lavender (primary button label, active segmented cell).
- **Line** (`rgba(233,233,237,0.16)` dark / `rgba(38,42,96,0.14)` light): Structural 1px borders — between desk bands, around controls.
- **Line Soft** (`rgba(233,233,237,0.10)` dark / `rgba(38,42,96,0.08)` light): Interior hairlines — table rows, the meta bar's lower edge, dialog header/footer rules, job separators in the queue.

### Tertiary — State
- **On Air Red** (`#ef5b6a` dark / `#c8283f` light): A render in progress, a job that *failed*, or a destructive action — the pulsing lamp bulb and its `RENDERING` / `RIPPING` / `ON AIR` word, failed queue rows and badges, the junk-match highlight, destructive-confirm text, error toasts, the one dot on the doctor strip.
- **Done Green** (`#5fd0a0` dark / `#20745a` light): Success only — the `Last —` queue item, a `Done` badge, the success toast dot.
- **Idle Grey** (`#3a3d5c` dark / `#d8d6ec` light): The lamp bulb at rest. The absence of signal.

### The One Colour Exception
The feed dialog's QR quiet zone is a literal `#fff` — a QR code must scan against
true white regardless of theme. It is the only hard-coded colour in the system;
everything else is a token.

### Named Rules
**The One Signal Rule.** Signal Lavender is the only chromatic accent in the
chrome. It marks exactly one primary action per view and the live-signal
surfaces (lamp, render rail, active segmented cell, focus). If a second passive
thing on screen is lavender, one of them is wrong.

**The Failure-Is-Red Rule.** Red is *earned by failure*: a job that failed, a
render in progress (a job that could fail), a destructive confirm. It is never
the colour of a warning or an unmet prerequisite. The RIP tab's "install yt-dlp"
strip is neutral `panel-raised` chrome carrying a single 6px `danger` dot — the
dot signals "attention", the card does not shout. Red arrives only when a rip
actually fails.

**The State-Colour Rule.** Red, green, and the lamp's idle grey are earned by
state, never chosen for decoration. Green always means "done". A control is never
red or green at rest.

**The Light-Theme Note.** `--muted` is the colour of the 10px engraved labels —
the smallest text in the app — so it carries the tightest contrast budget.
`#6b6d94` was 4.4:1 on `--bg` (an AA miss); the shipped `#5f6186` is 5.3:1, same
navy-lavender hue, one lightness step darker. `--ok` moved the same way
(`#2f8f6c` → `#20745a`). When adjusting a light-theme token, check it against
both `--bg` and white `--panel` and hold 4.5:1.

## Typography

**Chrome Font:** `ui-monospace` → Cascadia Mono → JetBrains Mono → Consolas → Liberation Mono → monospace
**Prose Font:** `system-ui` → Segoe UI → Roboto → sans-serif

**Character:** The monospace is console silkscreen — every label, button,
readout, and the `CLIP2POD` ident are set in it, uppercased and widely tracked so
the chrome reads as equipment marking. The system sans is deliberately plain and
quiet; it appears only where the user's own words or explanatory prose live, so
authored content never competes with the desk. Base size is a compact 14px.

### Hierarchy
- **Ident** (mono, 700, 15px, letter-spacing 0.22em): The `CLIP2POD` station mark in the header. The single largest, most-tracked element; appears once.
- **Body** (sans, 400, 13–14px, line-height 1.5): The article in the editor (13.5px) and paragraph copy inside dialogs. The only non-mono running text.
- **Control** (mono, 600, 11px, uppercase, letter-spacing 0.1em): Button faces. The `kbd` shortcut floats right on the same face at 9px, 0.72 opacity.
- **Label** (mono, 600, 10px, uppercase, letter-spacing 0.14em): Band and sidebar headers (`INTAKE`, `TRANSPORT`, `JUNK`, `TOOLS`, `ADD EPISODE`, `QUEUE`, `EPISODES` — real `<h2>` / `<h3>` carrying `.label`), field captions, table headers, dialog titles, settings subsections, the lamp's status word, segmented-cell faces.

### Supporting sizes
Below the primary roles, a fixed set of pixel steps recurs; keep new work on
these rather than inventing intermediate sizes.
- **`field` — 13px mono:** input text, dialog `code`, the `job-title` in the RIP queue.
- **`data` — 12.5px mono:** table cell text, toast body, coach-bar text, the junk-phrase textarea.
- **`note` — 12px sans:** dialog `.hint` / `.check` rows, the RIP doctor strip, the settings "detected version" line.
- **11–11.5px mono:** `.mono` path/data lines, queue-bar text, the "voices enabled" tally, the `fail-hint` under a failed rip.
- **`kbd` — 9px mono** and **`label` — 10px mono** are the floor.

### Named Rules
**The Two-Voice Rule.** Monospace is the chrome; system sans is the content.
There is no third face and no in-between use. If new text is a label, a control,
a status, or a number, it is mono and uppercase. If it is a sentence the user
reads for meaning, it is sans.

**The Silkscreen Rule.** All-caps mono chrome always carries tracking — 0.1em on
controls, 0.14em on labels, 0.22em on the ident. Uppercased mono without
letter-spacing is a bug.

## Layout

`overflow: hidden` on `html/body`; the app is a column at `100vh` — a shared
**header** over a **content area** that shows one mode's desk at a time. Both
modes' bodies stay mounted; the inactive one is `hidden`.

**Header** (`padding: 10px 16px`, panel, bottom `line`, `flex-wrap: wrap` with
`row-gap: 8px`): the `CLIP2POD` ident, then the **segmented mode toggle**
(`NARRATE` / `RIP`), the **combined status lamp**, and `FEED` / `LOG` /
`SETTINGS`. The three action buttons are ordinary `.btn`. It never wraps at
supported widths, but is allowed to if OS text scaling forces it.

### The NARRATE desk

Three bands filling the content area, no whole-surface scroll:

1. **Meta bar** (`padding: 12px 16px`, panel, bottom `line-soft`): a CSS grid, `grid-template-columns: 2fr 1.2fr 1.2fr auto` — Title, Author, Filename title, and the author-gender segmented control. Collapses to `1fr 1fr` below 900px.
2. **Deck** (`flex: 1`, `min-height: 0`): a flexible `main` (the CodeMirror editor, `min-width: 0`, its own `.cm-scroller`) beside a fixed **280px** sidebar (`border-left: 1px line`, its own `overflow-y: auto`). The coach bar floats absolutely centred 14px above the deck's bottom edge.
3. **Queue bar** (`padding: 8px 16px`, panel, top `line`): status label, a wrapping status strip (`max-height: 92px`, scrolls), right-aligned actions. A 2px render rail rides its top edge.

### The RIP desk

Two to four bands, same fixed frame:

1. **Doctor strip** (conditional, `panel-raised`, bottom `line`, `padding: 9px 16px`): shown only while `yt-dlp` / `ffmpeg` is missing. One wrapping row — a 6px `danger` dot, the one-line ask, the `winget` commands as `code`, a `Re-check` button.
2. **Add episode** (`band`): label + URL field + `Paste link` + `Download`.
3. **Queue** (conditional, `band`): the section head stays put; the job list is a `max-height: 168px` region with its own `overflow-y`, so a burst of jobs can never push the episode table off the desk.
4. **Episodes** (`band`, `flex: 1`, `min-height: 0`): the section head (`EPISODES — n`, Refresh, Delete all) stays put; the table sits in a `flex: 1` `.table-wrap` that scrolls, its `<thead>` sticky (`th { position: sticky; top: 0 }`). Empty state is a single muted line.

### The combined lamp

One lamp reads both workers. Bulb severity: **red** if either worker is
rendering, **lavender** if either has jobs queued, **grey** otherwise. The status
word names what's happening: `ON AIR` (both busy), `RENDERING` (narrate only),
`RIPPING` (rip only), `QUEUED n` (combined count), `IDLE`.

**Spacing rhythm:** a tight scale — `4px` (control gaps), `5px` (sidebar group
gap), `8px` (default gap / field padding), `10px` (sidebar padding), `12–16px`
(band padding). Sidebar groups drop `2px` under each label. The desk is
intentionally dense: an instrument panel, not a document.

**Responsive:** one breakpoint at **900px** — the NARRATE meta grid halves its
columns. The app is a desktop shell (Tauri); there is no phone layout. Under
`@media (pointer: coarse)` every `.btn` gets `min-height: 44px` and `.check` rows
widen their gap.

### Named Rules
**The Fixed-Desk Rule.** The window is always exactly the viewport, `overflow:
hidden`, and this holds for *both* tabs. New content finds room inside an
existing band's own scroll region — the editor, the sidebar, the queue strip, the
job list, the episode table, a dialog body. It never makes the window scroll and
never adds a fifth band.

**The Fixed Column Rule.** The NARRATE control column is exactly 280px (grown
from 240px once "Manage Author Genders" — the longest control label — needed
the room to keep its `kbd` shortcut on one line; the app's default window
width grew by the same 40px, to 1140px, so the editor didn't lose it).
Full-width buttons stacked in labelled groups; they do not reflow into a grid or
change width with the window.

## Elevation & Depth

The system is **tonally layered, not shadowed**. Depth comes entirely from the
three-step indigo ramp — `bg` (recessed: editor, fields) → `panel` (the equipment
band) → `panel-raised` (button and control faces, the doctor strip) — plus 1px
`line` / `line-soft` borders. Resting surfaces cast no shadow. In the light theme
the tonal steps nearly collapse and borders carry the separation alone.

Shadows appear **only on layers that genuinely float above the desk**: modals,
the coach bar, and toasts.

### Shadow Vocabulary
- **Floating panel** (`box-shadow: 0 8px 24px var(--shadow)`): The coach bar and toasts — a small element lifted just off the surface.
- **Modal lift** (`box-shadow: 0 18px 50px var(--shadow)`): The dialog — a larger throw for a larger overlay, over a `--shadow`-filled `::backdrop`.

`--shadow` is `rgba(10,10,20,0.55)` dark / `rgba(38,42,96,0.18)` light.

### Named Rules
**The Flat-Desk Rule.** Anything that is part of the fixed frame — header, desk
bands, sidebar, buttons, queue bar, tables, the doctor strip — is flat: tonal
fill plus border, zero shadow. Shadow is reserved for the three things that
overlay the desk.

## Shapes

The form language is **rectilinear and tight-cornered**. A three-step radius
scale, and nothing is round except the state markers.

- **`4px` (`sm`)** — the default. Buttons, fields, the status lamp housing, toasts, tables, the segmented control box, status badges, progress bars, code blocks, scrollbar thumbs, the QR frame.
- **`6px` (`md`)** — the coach bar only. A hair softer to read as a transient helper.
- **`8px` (`lg`)** — the modal only. The largest surface gets the largest corner.
- **`50%`** — state markers exclusively: the lamp bulb (10px), the toast dot (7px), the queue-item disc (6px), the doctor strip's dot (6px). Circles mean "status indicator" and nothing else.

Desk bands have **no radius** — they are full-width, butted together, divided by
1px `line`. A band is not a card. Borders are uniformly **1px** — `line` for
structural edges, `line-soft` for interior hairlines. The segmented control is
one bordered box with `overflow: hidden` and borderless internal cells divided by
nothing but their fill.

### Named Rules
**The Sharp-Corner Rule.** Radius never exceeds 8px, and only the modal reaches
it. Desk bands get no radius at all. If a new surface wants a rounder corner, it
is trying to look friendly — and this desk is equipment.

**The Band-Not-Card Rule.** Content on either desk lives in full-width bands
divided by `line`, never in inset rounded cards with gaps between them. A card
with a shadow or an 8px corner floating in a gutter is a different product.

**The Circle-Means-Status Rule.** A filled circle is always a state marker tinted
to its state (`idle` / `signal` / `danger` / `ok` / `accent`). Never a bullet, an
avatar frame, or decoration.

## Components

### Buttons
- **Shape:** 4px radius, 1px `line` border, mono/uppercase/0.1em face, `text-align: left`.
- **Default:** `panel-raised` fill, `text` colour, `padding: 7px 12px` (5px 10px in the sidebar and queue bar, 3px 8px for `.btn.small`). The `kbd` shortcut floats right at 9px / 0.72 opacity.
- **Primary:** `signal-lavender` fill, `accent-ink` text, border matches fill. Exactly one per view — `Generate MP3` on the NARRATE desk, `Download` on the RIP desk, the confirming action in a dialog footer.
- **Toggle:** a button reflecting on/off state (theme picker, author gender, mode toggle) carries `aria-pressed` — the fill colour is never the only signal.
- **Hover:** default → border shifts to `signal-lavender-dim` (fill unchanged); primary → `filter: brightness(1.08)`. Transition `border-color, background 120ms`.
- **Active:** `transform: translateY(1px)`.
- **Focus:** `outline: 2px solid var(--accent)`, `outline-offset: 1px` — shared by buttons, inputs, selects, textareas.
- **Disabled:** `opacity: 0.45`, `cursor: not-allowed`.

**The System-Button Rule.** Every clickable control is a `.btn`, a `.btn.small`,
or a segmented cell — never a bespoke button class. A one-off header or card
button skips the shared focus outline, the `pointer: coarse` 44px target, and the
hover treatment all at once; the merge fixed exactly this in the header.

### Inputs / Fields
- **Style:** `.field` — full-width, `bg` (recessed) fill, 1px `line` border, 4px radius, `padding: 6px 8px`, 13px text. Placeholder is `muted` at full opacity (a placeholder can be the field's only visible label, so it holds 4.5:1).
- **Focus:** the shared 2px accent outline. No glow, no border-colour animation.
- **Labelling:** a field with no adjacent `<label>` (the URL intakes, dialog search boxes, the junk textarea) carries an explicit `aria-label`. Where two like controls sit in one view — the two "Browse" buttons in Settings — each carries a distinguishing `aria-label`.

### Segmented Control (signature)
- **Style:** one box, 1px `line` border, 4px radius, `overflow: hidden`, fixed `height: 31px`. Cells are borderless mono/10px/uppercase, `panel-raised` fill, `muted` text; a `:focus-visible` cell gets an inset 2px accent outline.
- **State:** the active cell carries `aria-pressed="true"` and flips to `signal-lavender` fill / `accent-ink` text.
- **Recognized:** on the MetaBar's author-gender control, the active cell additionally gets a `2px solid currentColor` inset ring (`outline-offset: -3px`) when its value came from the recognized-authors table rather than the leftover sticky default — `currentColor` picks up `accent-ink` against the `signal-lavender` fill automatically, so no new colour token was needed for either theme. A `title` tooltip on the cell states the same thing for anyone who can't see the ring.
- **Uses:** the header mode toggle (`NARRATE` / `RIP`), author gender (`UNK` / `M` / `F`), and the model for any small either/or choice. It is **not** an ARIA tablist — it is a group of toggle buttons.

### Status Lamp (signature)
- A housing (`panel-raised`, 1px `line`, 4px radius, `min-width: 108px`) holding a 10px round **bulb** + a mono status word. One lamp reads **both** workers; an optional `label` prop supplies the word, else the lamp derives it from a single `Lamp` state.
- **Idle:** bulb `idle` grey, word `IDLE`, no glow.
- **Queued:** bulb + word `signal-lavender`, `box-shadow: 0 0 8px` halo, word `QUEUED n` (combined narrate + rip count).
- **On air:** bulb + word `danger` red, `box-shadow: 0 0 10px`, `onair-pulse` 1.4s ease-in-out infinite (halo shrinks + 0.75 opacity at the midpoint). Word is `RENDERING` (narrate), `RIPPING` (rip), or `ON AIR` (both). Reduced-motion drops the pulse for a steady 12px halo.
- The NARRATE queue bar's `Rendering` disc echoes this pulse on the same 1.4s cadence.

**The One-Lamp Rule.** There is exactly one status lamp and it always reads the
whole app — never one lamp per mode, never a second indicator. Its word, not a
second light, tells you which worker is live.

### Desk Band
- A full-width `panel` region, `padding: 12px 16px`, `border-bottom: 1px line`, **no radius**. Bands butt together with no gutter. The last band on a desk drops its bottom border and takes `flex: 1` to fill remaining height; its inner list or table owns the scroll.
- A band's head is a `.label` `<h2>`/`<h3>`, optionally with right-aligned actions in a `.section-head` flex row.

### Doctor Strip (RIP)
- A single wrapping row above the RIP intake band: `panel-raised` fill, `border-bottom: 1px line`, `padding: 9px 16px`, 12px sans. A 6px `danger` dot, then `text`-coloured copy leading with the fix ("Install yt-dlp…"), then the `winget` command(s) as `user-select: all` `code`, then a `Re-check` `.btn.small`.
- Shown only while `app.doctor` reports a missing tool. It informs; it does not block — narration is unaffected and a rip only fails when actually attempted.

### Queue Job Row (RIP) + Progress Bar
- A `.job` separated by `line-soft`: a status `.badge` (mono 9.5px uppercase — `muted` default, `accent` fill for Processing, `ok` border for Done, `danger` border for Failed), an ellipsised title, a `stage · speed` readout, and a `Stop` / `Why?` `.btn.small`.
- **Progress bar:** `height: 6px`, 4px radius, `panel-raised` track. A determinate `.fill` is a `signal-lavender` bar scaled by `transform: scaleX()` with a 300ms linear ease. An indeterminate `.fill` is a `-45deg` `accent` / `accent-dim` barber-pole sliding on `background-position` (`indeterminate-stripes` 0.7s linear infinite) so it never reads as "complete"; reduced-motion freezes the slide but keeps the stripes.
- A failed job's `Why?` reveals a `max-height: 120px` scrolling `<pre class="detail">` (mono 11px on `bg`) plus a one-line `fail-hint` about updating `yt-dlp`.

### Coach Bar (signature, NARRATE)
- An absolutely-positioned, horizontally-centred pill floating 14px above the deck's bottom edge: `panel-raised` fill, 1px `signal-lavender-dim` border, 6px radius, `padding: 8px 14px`, floating-panel shadow, `white-space: nowrap`. One line of status plus 2–4 inline `.btn` actions — the junk-review conversation, replacing itself as the review advances, clearing on `Escape`. Rises 4px + fades over 160ms; reduced-motion fades only.

### Dialogs
- A native `<dialog class="modal">` opened with `showModal()` — the browser owns the focus trap, `Esc`-to-close, focus return, background inerting.
- **Panel:** `panel` fill, 1px `line`, 8px radius, modal-lift shadow, `width: min(720px, 92vw)`, `max-height: 84vh`, `padding: 0`. Flex column: header / scrolling `.body` / optional footer, each `12px 16px` and divided by `line-soft`. Rises 6px + fades over 160ms; reduced-motion fades only.
- **Header:** an `<h2 class="label">` title linked by `aria-labelledby` + a `Close` button showing `Esc`. Settings subsections are `<h3 class="label">` (muted mono — never the accent).
- **Open focus is on the panel**, not the first control: `showModal()` then `dialogEl.focus()` on a `tabindex="-1"` dialog.
- **In-dialog feedback is inline, not a toast** — a `<dialog>` sits above the toast stack. Confirm on the control itself (the feed dialog's Copy → `Copied`; the log's disable-voice item → `Removed from rotation`).

**The Native-Dialog Rule.** Overlays that need protected focus use `<dialog>` +
`showModal()`, never a hand-rolled backdrop `<div>` with a JS focus trap — that
pattern shipped a real focus-escape bug. A confirm that doesn't need protected
focus uses the inline expand-in-place pattern (see Queue Bar).

### Context menu
- The log's right-click "disable this voice" menu is a `popover` (so it clears the top-layer `<dialog>` it lives in), positioned at the pointer, clamped with `min(…, calc(100vw - 220px))`. A capture-phase `Escape` closes only the menu; an outside `pointerdown` closes it too.

### Toasts
- Bottom-right stack, 8px gap. `panel-raised` fill, 1px `line`, 4px radius, `padding: 9px 14px 9px 28px`, floating-panel shadow, `max-width: 340px`. A 7px round marker at left — `ok` green by default, `danger` red for `data-kind="error"`, which also carries a visually-hidden `Error: ` prefix (`.vh`) so the kind isn't colour-only. Slides in 12px from the right over 180ms; reduced-motion fades. Live region: `role="status" aria-live="polite" aria-atomic="false"`.

### Tables
- Full-width, `border-collapse: collapse`, 12.5px. Headers mono/10px/uppercase/`muted`, `position: sticky; top: 0` on a `panel` fill, sortable via a bare `<button>` with an `accent` arrow. Cells `padding: 5px 8px`, `line-soft` row rules. Selected row: `color-mix(in srgb, var(--accent) 14%, transparent)`. Per-row action buttons carry a contextual `aria-label` ("Delete <title>").

### Editor (signature, NARRATE)
- CodeMirror 6 themed straight from the desk variables: `bg` ground, `text` ink, `var(--sans)` at 13.5px, `accent` caret, line wrapping on, no focus ring.
- **Selection:** `color-mix(in srgb, var(--accent) 22%, transparent)`.
- **Junk match:** the flagged phrase gets `color-mix(in srgb, var(--danger) 30%, transparent)` fill + a 1px `danger` outline + 2px radius; the whole line is selected and scrolled to centre.

## Do's and Don'ts

### Do:
- **Do** keep the window at exactly `100vh` with `overflow: hidden` on *both* tabs; give new content a scroll region inside an existing band (The Fixed-Desk Rule).
- **Do** lay both desks out as full-width `panel` bands divided by 1px `line`, no radius, no gutter (The Band-Not-Card Rule).
- **Do** make every clickable control a `.btn`, `.btn.small`, or segmented cell (The System-Button Rule).
- **Do** keep exactly one status lamp reading the whole app; let its word name the live worker (The One-Lamp Rule).
- **Do** set every label, button, status, and numeric readout in tracked uppercase monospace — 0.1em on controls, 0.14em on labels (The Silkscreen Rule).
- **Do** reserve system sans for the editor text and dialog prose, and nothing else (The Two-Voice Rule).
- **Do** limit Signal Lavender to one primary action per view plus the live-signal surfaces (The One Signal Rule).
- **Do** treat red as failure — a failed job, a render in flight, a destructive confirm. Signal a warning or unmet prerequisite with neutral chrome plus one `danger` state dot (The Failure-Is-Red Rule).
- **Do** build depth from the `bg` → `panel` → `panel-raised` tonal ramp plus 1px borders; leave resting surfaces flat (The Flat-Desk Rule).
- **Do** render state markers as `50%`-radius circles tinted to `idle` / `signal` / `danger` / `ok` (The Circle-Means-Status Rule).
- **Do** give every NARRATE primary action a visible `kbd` shortcut on its own face.
- **Do** handle destructive confirmation inline (the queue bar's expand-in-place), first focus on the safe choice, focus returned to the trigger on close.
- **Do** keep radii at `4px` for chrome, `6px` for the coach bar, `8px` for modals — and no radius on desk bands (The Sharp-Corner Rule).
- **Do** drop looping and spatial motion under `prefers-reduced-motion`, but keep colour and opacity transitions that carry state.
- **Do** reach for a native `<dialog>` + `showModal()` for any overlay needing protected focus (The Native-Dialog Rule); move focus to the panel, confirm in-dialog actions inline.
- **Do** render a transient overlay that appears *inside* an open dialog (a context menu, a picker) as a `popover`, clamped to the viewport.
- **Do** give every colour-only signal a text equivalent — `aria-pressed` on toggles, the `.vh` `Error:` prefix, contextual `aria-label`s on repeated row actions.
- **Do** duplicate the full light palette under `@media (prefers-color-scheme: light) { :root:not([data-theme]) { … } }` so the pre-JS first paint matches the OS.
- **Do** hold every light-theme token at 4.5:1 against **both** `--bg` and white `--panel` (The Light-Theme Note).

### Don't:
- **Don't** let either tab scroll as a whole surface, or add a fifth band to a desk.
- **Don't** wrap desk content in inset rounded cards with gutters between them (The Band-Not-Card Rule).
- **Don't** create a bespoke button class for the header, a card, or anywhere else (The System-Button Rule).
- **Don't** add a second status indicator, or split the lamp per mode (The One-Lamp Rule).
- **Don't** use red for a warning, a hint, or a missing dependency — red is for something that failed (The Failure-Is-Red Rule).
- **Don't** put a shadow on any part of the fixed frame — only modals, the coach bar, and toasts float (The Flat-Desk Rule).
- **Don't** introduce a second accent hue, or use red/green on a control at rest (The State-Colour Rule).
- **Don't** set chrome text in the sans font, or the editor/prose text in mono (The Two-Voice Rule).
- **Don't** use uppercased monospace without letter-spacing.
- **Don't** colour a settings section header, or any passive label, in the accent — labels are `muted` mono.
- **Don't** use a filled circle as a bullet, avatar frame, or decoration (The Circle-Means-Status Rule).
- **Don't** widen or reflow the 280px NARRATE control column.
- **Don't** exceed an 8px corner radius, and don't reach 8px anywhere but the modal (The Sharp-Corner Rule).
- **Don't** open a modal for a destructive confirm when an inline expand fits.
- **Don't** hand-roll a modal backdrop `<div>` with a JS focus trap (The Native-Dialog Rule).
- **Don't** build an ARIA `tablist` for the mode toggle — it's a group of `aria-pressed` toggle buttons, not tabs.
- **Don't** dim placeholder text with extra `opacity`; `--muted` already fails contrast if dimmed further.
- **Don't** add decorative motion; motion is for state only.
