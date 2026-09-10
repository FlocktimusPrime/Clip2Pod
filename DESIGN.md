---
name: Clip2Pod
description: A one-person broadcast desk that turns copied text into a private podcast feed.
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
  headline:
    fontFamily: "ui-monospace, 'Cascadia Mono', 'JetBrains Mono', Consolas, monospace"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "0.22em"
  body:
    fontFamily: "system-ui, 'Segoe UI', Roboto, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
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
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
spacing:
  sm: "4px"
  md: "8px"
  lg: "10px"
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
  field:
    backgroundColor: "{colors.bg}"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    padding: "6px 8px"
  segmented-cell:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    padding: "0 12px"
    height: "31px"
  segmented-cell-active:
    backgroundColor: "{colors.signal-lavender}"
    textColor: "{colors.accent-ink}"
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
---

# Design System: Clip2Pod

## Overview

**Creative North Star: "The Production Desk"**

Clip2Pod is a one-person broadcast console. The window is a fixed, non-scrolling
desk: a header rail with the station ident and a status lamp, a metadata strip,
a large script area with a control column beside it, and a queue rail pinned to
the bottom edge. Every part of the frame is a piece of equipment — the lamp
tells you whether the desk is live, the transport buttons move a script through
intake → clean → render, the rail under the queue bar charges left-to-right as
audio is cut. Nothing on the desk is decorative; if an element is visible, it
either reports state or performs an action.

The palette is a dark studio by default — deep indigo panels stacked by
lightness, one soft lavender that means *signal* (the lamp when queued, the
primary action, the render rail) and one red that means *on air* (the pulsing
lamp during a render). Type does the rest of the work: a monospace voice for
every control, label, and data readout, uppercased and letter-spaced like
console silkscreen, with a plain system sans reserved for the two places real
prose lives — the article in the editor and the body copy inside dialogs. The
result is dense, calm, and legible at a glance, the way a mixing desk is: you
learn the surface once and then operate it without looking.

This is a keyboard-first instrument. Every primary action shows its shortcut on
its own face (`Ctrl+L`, `Ctrl+Enter`), and the whole app can be summoned and fed
from a global hotkey. The desk favours a settled, equipment-grade restraint over
expression — motion is limited to state (a lamp pulse, a rail easing toward the
next chunk, a dialog rising 6px) and never to ornament.

**Key Characteristics:**
- A fixed, four-band desk layout that never scrolls as a whole — only its inner regions do.
- Monospace, uppercase, letter-spaced type for all chrome; system sans only for authored prose.
- One accent (Signal Lavender) for *signal*, one red for *on air*, everything else neutral indigo.
- Tonal depth: surfaces separate by lightness step, not by shadow. Shadows mark only floating layers.
- A single circular-marker vocabulary — the lamp bulb, the toast dot, the queue-item disc — tinted to state.
- Every control wears its keyboard shortcut.

## Colors

A dark indigo studio with a single cool accent; the light theme is the same room
with the lights on — near-white panels on a lilac-grey floor, the accent
darkened to hold contrast.

### Primary
- **Signal Lavender** (`#b5abfc` dark / `#5b4fc4` light): The one accent. Used for the *live signal* — the status lamp when a job is queued, the `Generate MP3` primary button, the render-progress rail, the editor caret and selection, focus outlines, sort arrows, checkbox/`accent-color`. It is never used as a surface fill for passive content.
- **Signal Lavender Dim** (`#6f63b0` dark / `#b5abfc` light): The muted echo of the accent — button hover borders, scrollbar thumbs, the coach bar's border. Signals "interactive, not active".

### Neutral
- **Desk Black** (`#161826` dark / `#f2f1fa` light): The window ground and the editor/field background — the deepest layer, where authored text sits.
- **Panel** (`#1d1f3d` dark / `#ffffff` light): The equipment surface — header, meta bar, sidebar, queue bar, dialog body. One step up from the ground.
- **Panel Raised** (`#262a60` dark / `#ffffff` light): The top tonal layer — button faces, the lamp housing, segmented cells, toasts, the coach bar. In light theme it collapses to plain white and relies on borders.
- **Ink** (`#e9e9ed` dark / `#1f2140` light): Primary text.
- **Muted Ink** (`#9b9bb8` dark / `#5f6186` light): Labels, taglines, placeholders, secondary and "waiting" queue items.
- **Accent Ink** (`#161826` dark / `#ffffff` light): Text that sits *on* Signal Lavender (primary button label, active segmented cell).
- **Line** (`rgba(233,233,237,0.16)` dark / `rgba(38,42,96,0.14)` light): Structural 1px borders between bands and around controls.
- **Line Soft** (`rgba(233,233,237,0.10)` dark / `rgba(38,42,96,0.08)` light): Interior hairlines — table rows, the meta bar's lower edge, dialog header/footer rules.

### Tertiary — State
- **On Air Red** (`#ef5b6a` dark / `#c8283f` light): Render-in-progress only — the pulsing lamp bulb and `RENDERING` text, failed queue rows, the junk-match highlight, destructive-confirm text, error toasts and banners.
- **Done Green** (`#5fd0a0` dark / `#20745a` light): Success only — the `Last —` queue item, the success toast dot.
- **Idle Grey** (`#3a3d5c` dark / `#d8d6ec` light): The lamp bulb at rest. The absence of signal.

### The One Colour Exception
The feed dialog's QR quiet zone is a literal `#fff` — a QR code must scan against
true white regardless of theme. It is the only hard-coded colour in the system;
everything else is a token.

### Named Rules
**The One Signal Rule.** Signal Lavender is the only chromatic accent in the
chrome. It marks exactly one primary action per view and the live-signal
surfaces (lamp, render rail, focus). If a second thing on screen is lavender,
one of them is wrong.

**The State-Colour Rule.** Red, green, and the lamp's idle grey are *earned by
state*, never chosen for decoration. Red always means "rendering / failed /
destructive". Green always means "done". A control is never red or green at rest.

**The Light-Theme Note.** `--muted` is the colour of the 10px engraved labels —
the smallest text in the app — so it carries the tightest contrast budget.
`#6b6d94` was 4.4:1 on `--bg` (an AA miss); the shipped `#5f6186` is 5.3:1, same
navy-lavender hue, one lightness step darker. `--ok` moved the same way
(`#2f8f6c` → `#20745a`, 4.0:1 → 5.7:1 on white). When adjusting a light-theme
token, check it against both `--bg` and `--panel` (white) and hold 4.5:1.

## Typography

**Display / Chrome Font:** `ui-monospace` → Cascadia Mono → JetBrains Mono → Consolas → Liberation Mono → monospace
**Body Font:** `system-ui` → Segoe UI → Roboto → sans-serif

**Character:** The monospace is console silkscreen — every label, button,
readout, and the `CLIP2POD` ident are set in it, uppercased and widely tracked
so the chrome reads as equipment marking. The system sans is deliberately plain
and quiet; it appears only where the user's own words or explanatory prose live,
so authored content never competes with the desk. Base size is a compact 14px.

### Hierarchy
- **Headline / Ident** (mono, 700, 15px, letter-spacing 0.22em): The `CLIP2POD` station mark in the header. The single largest, most-tracked element; appears once.
- **Body** (sans, 400, 13–14px, line-height 1.5): The article in the editor (13.5px) and paragraph copy inside dialogs. The only non-mono text.
- **Control** (mono, 600, 11px, uppercase, letter-spacing 0.1em): Button faces. The `kbd` shortcut floats right on the same face at 9px, 0.72 opacity.
- **Label** (mono, 600, 10px, uppercase, letter-spacing 0.14em): Section headers in the sidebar (`INTAKE`, `TRANSPORT`, `JUNK`, `TOOLS` — real `<h2>` elements carrying the `.label` class), field captions in the meta bar, table headers, dialog titles (`<h2>`), settings subsections (`<h3>`), the lamp's status word.
- **Mono Data** (mono, 400, 11–12.5px): Queue-bar items, the "voices enabled" tally, the feed URL, table cell text — dense readouts that benefit from fixed advance width.

### Supporting sizes
Below the four frontmatter roles, a handful of pixel steps recur in component
styles. They are the same set yt-dlFeed uses; keep new work on these rather than
inventing intermediate sizes:
- **13px** — `.field` input text, dialog `code`.
- **12.5px** — table cells, toast body, coach-bar text, the junk-phrase textarea.
- **12px** — dialog `.hint` / `.check` rows, the header tagline (11.5px).
- **11–11.5px** — `.mono` path/data lines, queue-bar text, sidebar readouts.
- **9–10px** — the `kbd` shortcut on a button (9px), all `.label` chrome (10px).

### Named Rules
**The Two-Voice Rule.** Monospace is the chrome; system sans is the content.
There is no third face and no in-between use. If new text is a label, a control,
a status, or a number, it is mono and uppercase. If it is a sentence the user
reads for meaning, it is sans.

**The Silkscreen Rule.** All-caps mono chrome always carries tracking — 0.1em on
controls, 0.14em on labels, 0.22em on the ident. Uppercased mono without letter-
spacing is a bug.

## Layout

The window is a **fixed four-band vertical desk** at `100vh`, `overflow: hidden`
on `html/body` — the app frame never scrolls; only inner regions do.

1. **Header** (`padding: 10px 16px`, panel, bottom `line`): ident + tagline on the left (tagline hides below 900px), status lamp, `FEED` button on the right.
2. **Meta bar** (`padding: 12px 16px`, panel, bottom `line-soft`): a CSS grid, `grid-template-columns: 2fr 1.2fr 1.2fr auto` — Title, Author, Filename title, and the author-gender segmented control. Collapses to `1fr 1fr` below 900px.
3. **Deck** (`flex: 1`, `min-height: 0`): the working area — a flexible `main` (the CodeMirror editor, `min-width: 0`) beside a fixed **240px** sidebar (`border-left: 1px line`, its own `overflow-y: auto`). The coach bar floats absolutely centred 14px above the deck's bottom edge.
4. **Queue bar** (`padding: 8px 16px`, panel, top `line`): status label, a wrapping status strip (`max-height: 92px`, scrolls), and right-aligned actions. A 2px render rail rides its top edge.

**Spacing rhythm:** a tight scale — `4px` (control gaps), `8px` (default gap / field padding), `10px` (sidebar padding and group gap), `12–16px` (band padding). Sidebar groups stack at `5px` internal gap with a `2px` drop under each label. The desk is intentionally dense: this is an instrument panel, not a document.

**Responsive:** one breakpoint at **900px** — the header tagline disappears and
the meta grid halves its columns. The desk is a desktop application shell (Tauri);
there is no phone layout.

### Named Rules
**The No-Scroll-Frame Rule.** The outer desk is always exactly the viewport.
New content finds room inside an existing band's own scroll region (the editor,
the sidebar, the queue strip, a dialog body) — it never makes the window scroll
and never adds a fifth band.

**The Fixed Column Rule.** The control column is exactly 240px. Controls are
full-width buttons stacked in labelled groups; they do not reflow into a grid or
change width with the window.

## Elevation & Depth

The system is **tonally layered, not shadowed**. Depth on the desk comes
entirely from the three-step indigo ramp — `bg` (recessed: editor, fields) →
`panel` (the equipment surface) → `panel-raised` (button and control faces) —
plus 1px `line` / `line-soft` borders. Resting surfaces cast no shadow. In the
light theme the tonal steps nearly collapse (white on white) and borders carry
the separation alone.

Shadows appear **only on layers that genuinely float above the desk**: modals,
the coach bar, and toasts. Their shadow is soft, large, and low-contrast — it
says "this is temporarily on top", not "this is a card".

### Shadow Vocabulary
- **Floating panel** (`box-shadow: 0 8px 24px var(--shadow)`): The coach bar and toasts — a small element lifted just off the surface.
- **Modal lift** (`box-shadow: 0 18px 50px var(--shadow)`): The dialog — a larger throw for a larger, more disruptive overlay, over a `--shadow`-filled `::backdrop`.

`--shadow` is `rgba(10,10,20,0.55)` dark / `rgba(38,42,96,0.18)` light.

### Named Rules
**The Flat-Desk Rule.** Anything that is part of the fixed frame — header, meta
bar, sidebar, buttons, queue bar, tables — is flat: tonal fill plus border, zero
shadow. A shadow on a resting surface is a mistake. Shadow is reserved for the
three things that overlay the desk.

## Shapes

The form language is **rectilinear and tight-cornered**. There is a three-step
radius scale and nothing is round except the state markers.

- **`4px` (`sm`)** — the default. Buttons, fields, the status lamp housing, toasts, tables, the segmented control, code blocks, scrollbar thumbs, the QR frame. Almost every rectangle on the desk.
- **`6px` (`md`)** — the coach bar only. A hair softer to read as a transient helper distinct from permanent chrome.
- **`8px` (`lg`)** — the modal only. The largest surface gets the largest corner.
- **`50%`** — state markers exclusively: the lamp bulb (10px), the toast dot (7px), the queue-item disc (6px). Circles mean "status indicator" and nothing else.

Borders are uniformly **1px** — `line` for structural edges (between bands,
around controls), `line-soft` for interior hairlines (table rows, dialog
header/footer). The segmented control is one bordered box with `overflow: hidden`
and borderless internal cells divided by nothing but their fill.

### Named Rules
**The Sharp-Corner Rule.** Radius never exceeds 8px, and only the modal reaches
it. If a new surface wants a rounder corner, it is trying to look friendly — and
this desk is equipment, not a greeting card.

**The Circle-Means-Status Rule.** A filled circle is always a state marker tinted
to its state (`idle` / `signal` / `danger` / `ok` / `accent`). Never use a disc
as a bullet, an avatar frame, or decoration.

## Components

### Buttons
- **Shape:** 4px radius (`sm`), 1px `line` border, mono/uppercase/0.1em face, `text-align: left`.
- **Default:** `panel-raised` fill, `text` colour, `padding: 7px 12px` (5px 10px in the sidebar and queue bar). The `kbd` shortcut floats right at 9px / 0.72 opacity.
- **Toggle:** a button that reflects on/off state (theme picker, author-gender cells) carries `aria-pressed` alongside the `.primary` / `.active` fill — the colour is not the only signal.
- **Touch:** under `@media (pointer: coarse)` every `.btn` gets `min-height: 44px`; the mouse layout is untouched.
- **Primary:** `signal-lavender` fill, `accent-ink` text, border matches fill. Exactly one per view — `Generate MP3` on the main desk, the confirming action in a dialog footer.
- **Hover:** default → border shifts to `signal-lavender-dim` (fill unchanged); primary → `filter: brightness(1.08)`. Transition `border-color, background 120ms`.
- **Active:** `transform: translateY(1px)` — a physical key press.
- **Focus:** `outline: 2px solid var(--accent)`, `outline-offset: 1px` — shared by buttons, inputs, selects, textareas.
- **Disabled:** `opacity: 0.45`, `cursor: not-allowed`.

### Inputs / Fields
- **Style:** `.field` — full-width, `bg` (recessed) fill, 1px `line` border, 4px radius, `padding: 6px 8px`, 13px text. Placeholder is `muted` at full opacity (no extra dimming — a placeholder can be the field's only visible label, so it holds 4.5:1).
- **Focus:** the shared 2px accent outline. No glow, no border-colour animation.
- **Context:** the URL intake field, all three meta-bar fields, dialog search boxes, the junk-phrase textarea.
- **Labelling:** a field with no adjacent `<label>` (the URL intake, dialog search boxes, the junk textarea) carries an explicit `aria-label`; the meta-bar fields are wrapped in their `<label>`.

### Segmented Control
- **Style:** one box, 1px `line` border, 4px radius, `overflow: hidden`, fixed `height: 31px`. Cells are borderless mono/10px/uppercase, `panel-raised` fill, `muted` text.
- **State:** the active cell flips to `signal-lavender` fill / `accent-ink` text. Used for author gender (`UNK` / `M` / `F`) and as the model for any small either/or choice.

### Status Lamp (signature)
- A housing (`panel-raised`, 1px `line`, 4px radius, `min-width: 108px`) holding a 10px round **bulb** + a mono status word.
- **Idle:** bulb `idle` grey, word `IDLE`, no glow.
- **Queued:** bulb + word `signal-lavender`, `box-shadow: 0 0 8px` halo, word `QUEUED n`.
- **On air:** bulb + word `danger` red, `box-shadow: 0 0 10px`, `onair-pulse` 1.4s ease-in-out infinite (halo shrinks + 0.75 opacity at the midpoint), word `RENDERING`. Reduced-motion drops the pulse for a steady 12px halo.
- The queue bar's `Rendering` item disc echoes this pulse on the same 1.4s cadence — one "desk is live" heartbeat in two places.

### Queue Bar (signature)
- A bottom `footer`, mono/11px, with a 2px **render rail** on its top edge: `background: signal-lavender`, `transform: scaleX(--p)` from `--p: 0..1`, eased `transform 400ms cubic-bezier(0.16,1,0.3,1)` so discrete chunk jumps read as a continuous charge.
- Status items each hang a 6px `currentColor` disc in a fixed left gutter: `muted` (idle / waiting), `accent` (rendering), `danger` (failed, text never truncated), `ok` (last done).
- The destructive `Delete episodes` action expands inline into a confirm row (`Delete n episodes?` in red + `Delete` / `Keep`), focus landing on the safe `Keep` and returning to the trigger on close — never a modal.

### Coach Bar (signature)
- An absolutely-positioned, horizontally-centred pill floating 14px above the deck's bottom edge: `panel-raised` fill, 1px `signal-lavender-dim` border, 6px radius, `padding: 8px 14px`, floating-panel shadow, `white-space: nowrap`.
- Carries one line of status text plus 2–4 inline `.btn` actions. It is the junk-review conversation — it replaces itself as the review advances and clears on `Escape`. Rises 4px + fades in over 160ms (`cubic-bezier(0.16,1,0.3,1)`); reduced-motion fades only.

### Dialogs
- **Element:** a native `<dialog class="modal">` opened with `showModal()`. The browser owns the focus trap, `Esc`-to-close, focus return to the opener, and background inerting — no hand-rolled `trapTab`.
- **Panel:** `.modal` — `panel` fill, 1px `line`, 8px radius, modal-lift shadow, `width: min(720px, 92vw)`, `max-height: 84vh`, `padding: 0`. `.modal[open]` is the flex column: header / scrolling `.body` / optional footer, each `padding: 12px 16px` and divided by `line-soft`. Rises 6px + fades over 160ms on open; reduced-motion fades only.
- **Backdrop:** `.modal::backdrop` — `--shadow` fill, fade-in 120ms.
- **Header:** an `<h2 class="label">` title linked by `aria-labelledby` + a `Close` button showing `Esc`. Settings subsections are `<h3 class="label">`.
- **Behaviour:** `Esc`, backdrop click, and the `Close` button all route through `onclose`. Don't add `role="dialog"` / `aria-modal` — `showModal()` implies them.
- **In-dialog feedback is inline, not a toast.** A `<dialog>` sits in the top layer *above* the toast stack, so a toast fired from inside it is hidden. Confirm an action on the control itself (the feed dialog's Copy → `Copied`).

**The Native-Dialog Rule.** New overlays that need protected focus use `<dialog>` + `showModal()`, never a hand-rolled backdrop `<div>` with a JS focus trap. A confirm that doesn't need protected focus uses the inline expand-in-place pattern instead (see Queue Bar).

### Toasts
- Bottom-right stack, 8px gap. `panel-raised` fill, 1px `line`, 4px radius, `padding: 9px 14px 9px 28px`, floating-panel shadow, `max-width: 340px`.
- A 7px round marker at left, `ok` green by default, `danger` red for `data-kind="error"`. An error toast also carries a visually-hidden `Error: ` prefix (via `.vh`) so the kind isn't colour-only. Slides in 12px from the right over 180ms; reduced-motion fades in place.
- Live region: `role="status" aria-live="polite" aria-atomic="false"` on the stack container.

### Tables
- Full-width, `border-collapse: collapse`, 12.5px. Headers are mono/10px/uppercase/`muted`, `position: sticky; top: 0` on a `panel` fill, sortable via a bare `<button>` with an `accent`-coloured arrow. Cells `padding: 5px 8px`, `line-soft` row rules. Selected row: `background: color-mix(in srgb, var(--accent) 14%, transparent)`.

### Editor (signature)
- CodeMirror 6 themed straight from the desk variables: `bg` ground, `text` ink, `var(--sans)` at 13.5px, `accent` caret, line wrapping on, 10px/12px padding, no focus ring (`&.cm-focused { outline: none }`).
- **Selection:** `color-mix(in srgb, var(--accent) 22%, transparent)`.
- **Junk match:** the flagged phrase gets `color-mix(in srgb, var(--danger) 30%, transparent)` fill + a 1px `danger` outline + 2px radius; the whole line is selected and scrolled to centre.

## Do's and Don'ts

### Do:
- **Do** keep the outer frame at exactly `100vh` with `overflow: hidden`; give new content a scroll region inside an existing band (The No-Scroll-Frame Rule).
- **Do** set every label, button, status, and numeric readout in tracked uppercase monospace — 0.1em on controls, 0.14em on labels (The Silkscreen Rule).
- **Do** reserve system sans for the editor text and dialog prose, and nothing else (The Two-Voice Rule).
- **Do** limit Signal Lavender to one primary action per view plus the live-signal surfaces — lamp, render rail, focus outline, caret (The One Signal Rule).
- **Do** build depth from the `bg` → `panel` → `panel-raised` tonal ramp plus 1px borders; leave resting surfaces flat (The Flat-Desk Rule).
- **Do** give every primary action a visible `kbd` shortcut on its own face.
- **Do** render state markers as `50%`-radius circles tinted to `idle` / `signal` / `danger` / `ok` (The Circle-Means-Status Rule).
- **Do** handle destructive confirmation inline (the queue bar's expand-in-place pattern), keeping first focus on the safe choice and returning it to the trigger on close.
- **Do** keep radii at `4px` for chrome, `6px` for the coach bar, `8px` for modals — and no further (The Sharp-Corner Rule).
- **Do** drop looping and spatial motion under `prefers-reduced-motion`, but keep colour and opacity transitions that carry state.
- **Do** reach for a native `<dialog>` + `showModal()` for any overlay that needs protected focus (The Native-Dialog Rule).
- **Do** give every colour-only signal a text equivalent — `aria-pressed` on toggle buttons, the `.vh` `Error:` prefix on error toasts.
- **Do** duplicate the full light palette under `@media (prefers-color-scheme: light) { :root:not([data-theme]) { … } }` so the pre-JS first paint matches the OS; an explicit `data-theme` still wins on specificity.
- **Do** hold every light-theme token at 4.5:1 against **both** `--bg` and white `--panel` (The Light-Theme Note).

### Don't:
- **Don't** put a shadow on any part of the fixed desk — only modals, the coach bar, and toasts float (The Flat-Desk Rule).
- **Don't** introduce a second accent hue, or use red/green on a control at rest (The State-Colour Rule).
- **Don't** set chrome text in the sans font, or the editor/prose text in mono (The Two-Voice Rule).
- **Don't** use uppercased monospace without letter-spacing.
- **Don't** use a filled circle as a bullet, avatar frame, or decoration (The Circle-Means-Status Rule).
- **Don't** widen or reflow the 240px control column, or add a fifth band to the desk.
- **Don't** exceed an 8px corner radius, and don't reach 8px anywhere but the modal (The Sharp-Corner Rule).
- **Don't** open a modal for a destructive confirm when an inline expand fits.
- **Don't** hand-roll a modal backdrop `<div>` with a JS focus trap — that pattern shipped a real bug (focus escaping to a disabled button) and is why the app moved to native `<dialog>`.
- **Don't** dim placeholder text with extra `opacity`; `--muted` is already the lighter tier and further dimming fails contrast.
- **Don't** add decorative motion; motion is for state (lamp pulse, render rail, overlay entrance) only.
