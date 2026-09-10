# Design reconciliation — Clip2Pod ↔ yt-dlFeed

> **Resolved 2026-09 (v0.7.0).** yt-dlFeed was merged into Clip2Pod as the **RIP**
> tab; `ytdlfeed-core` is now a vendored crate in this repo and the standalone
> yt-dlFeed repo is archived. The "shared design system" is no longer two files
> kept in sync — it is one `theme.css` and one `src/lib/components/` set that both
> tabs import. There is nothing left to port *between* apps. The checklist below is
> kept for history and because the shared-layer a11y/responsive items it lists
> still apply to the merged app; run `/impeccable audit` against `DESIGN.md` to
> re-check them.

**Written:** 2026-09-09, from a design pass on the then-sibling app **yt-dlFeed**.

Clip2Pod and yt-dlFeed share one design system — North Star **"The Production
Desk"**, one palette, one type system, one set of Named Rules. The shared
surface is `theme.css` plus the chrome components `Modal`, `StatusLamp`,
`Toasts`, `StartupPromptDialog` (and the `.btn` / `.field` / `.modal` / table
primitives inside `theme.css`).

A full `/impeccable audit` was run on yt-dlFeed and all findings remediated over
9 passes. Several fixes belong in the shared layer and should land in Clip2Pod
too. A few things Clip2Pod already does better should flow the other way. This
document is the checklist for that reconciliation.

---

## How to run the catch-up

1. Open a session in `C:\Users\ketov\Projects\Clip2Pod`.
2. Point it at this file, then run **`/impeccable audit`**. It now reads
   `Clip2Pod/DESIGN.md`, so it will surface Clip2Pod's own a11y / responsive /
   theming findings — many overlap with yt-dlFeed's original audit (unlabeled
   inputs, heading hierarchy, the hand-rolled modal, the `--muted` contrast
   miss).
3. Work the audit's recommended commands the same way yt-dlFeed did — one at a
   time or all at once. Use the **Port into Clip2Pod** section below as the
   concrete diff for the shared-layer items, so you're not re-deriving them.
4. After the shared files change, apply the **Port into yt-dlFeed** items back
   here (`C:\Users\ketov\Projects\yt-dlFeed`) so the two don't re-diverge.
5. Re-run `/impeccable audit` in both repos to confirm.

A one-line kickoff prompt for the Clip2Pod session:

> Read `docs/design-reconciliation.md`, then run `/impeccable audit`. Treat the
> "Port into Clip2Pod" section as already-decided work — apply those shared-layer
> changes, then address whatever else the audit finds.

---

## Port into Clip2Pod (yt-dlFeed is ahead)

### 1. Light `--muted` contrast — WCAG AA fix

`src/lib/theme.css:28` — `--muted: #6b6d94` → **`#5f6186`**

`#6b6d94` is **4.4:1** on `--bg` (`#f2f1fa`) and **4.95:1** on `--panel` — the
first is an AA failure, and `--muted` is the colour of the 10px engraved labels,
the smallest text in the app. `#5f6186` is 5.3:1 / 5.9:1, same navy-lavender
hue, one lightness step darker.

Also update:
- `DESIGN.md:148` — the Muted Ink light value.
- `.impeccable/design.json` → `extensions.colorMeta.muted.light`.
- Add the rationale as a note (yt-dlFeed's `DESIGN.md` carries it as **The
  Light-Theme Note** under Colors).

### 2. Native `<dialog>` modal

`src/lib/components/Modal.svelte` is the hand-rolled backdrop + `trapTab` +
`$effect(focus)` version. It has a real bug: `trapTab` collects `:disabled`
elements, so when the last focusable is a disabled footer button (e.g. Settings
"Save" before the form is dirty), forward-Tab calls `.focus()` on a disabled
node and focus escapes the dialog.

Replace with a native `<dialog>` opened via `showModal()` — the browser then
supplies the focus trap, `Esc`-to-close, focus return to the opener, background
inerting, and `::backdrop`. yt-dlFeed's version (post-change):

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";
  let { title, onclose, children, footer }:
    { title: string; onclose: () => void; children: Snippet; footer?: Snippet } = $props();
  let dialogEl: HTMLDialogElement;
  const titleId = `modal-title-${Math.random().toString(36).slice(2)}`;
  $effect(() => { dialogEl.showModal(); });
</script>

<dialog
  class="modal"
  bind:this={dialogEl}
  aria-labelledby={titleId}
  onclose={onclose}
  onclick={(e) => { if (e.target === dialogEl) onclose(); }}
>
  <header>
    <h2 class="label" id={titleId}>{title}</h2>
    <button class="btn" onclick={onclose}>Close <kbd>Esc</kbd></button>
  </header>
  <div class="body">{@render children()}</div>
  {#if footer}<footer>{@render footer()}</footer>{/if}
</dialog>
```

`theme.css` changes that go with it (replace the `.modal-backdrop` + `.modal`
block):

```css
.modal {
  background: var(--panel);
  color: var(--text);
  border: 1px solid var(--line);
  border-radius: 8px;
  box-shadow: 0 18px 50px var(--shadow);
  width: min(720px, 92vw);
  max-height: 84vh;
  padding: 0;
  overflow: hidden;
}
.modal[open] { display: flex; flex-direction: column; }
.modal::backdrop { background: var(--shadow); }
```

**Keep Clip2Pod's entrance animation.** Native `<dialog>` can still animate —
move the `modal-in` / `backdrop-in` keyframes onto `.modal[open]` and
`.modal::backdrop`, and gate on `@starting-style` for the enter transition, or
keep the CSS `animation` on `.modal[open]` (it fires when the element is shown).
Preserve the existing `@media (prefers-reduced-motion)` fallback that swaps the
6px rise for a plain fade.

Any component-level Escape handler in `+page.svelte` (yt-dlFeed had
`if (e.key === "Escape") app.dialog = null` on `<svelte:window>`) can be
deleted — the native dialog fires `close` → `onclose`.

Because `onclose` now runs on Escape too, `StartupPromptDialog` will correctly
route Escape through `answer(false)` instead of leaving the setting unset.

### 3. `prefers-color-scheme` fallback

`theme.css` — before JS sets `data-theme`, the app is dark. On a light-scheme OS
with a saved light theme, the first paint flashes dark. Add, right after the
`:root[data-theme='light']` block:

```css
@media (prefers-color-scheme: light) {
  :root:not([data-theme]) {
    /* full light palette, duplicated — an explicit data-theme still wins */
  }
}
```

`:root:not([data-theme])` matches only during the pre-JS window; the instant
`initApp()` stamps `data-theme`, the explicit block takes over.

### 4. Toast — non-colour error indication

`src/lib/components/Toasts.svelte` — the error/success distinction is carried
only by the `::before` dot colour. Add a visually-hidden prefix and a `.vh`
utility (yt-dlFeed added `.vh` to `theme.css`):

```svelte
<div class="toast" data-kind={t.kind}
  >{#if t.kind === "error"}<span class="vh">Error: </span>{/if}{t.text}</div
>
```

```css
.vh {
  position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
  overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; border: 0;
}
```

Clip2Pod's toast live region (`role="status" aria-live="polite"
aria-atomic="false"`) is already correct — **standardise both apps on it**;
change yt-dlFeed's bare `aria-live="polite"` to match this exact attribute set.

### 5. Touch targets

`theme.css` — Clip2Pod is also a Tauri desktop shell that can run on a
touchscreen laptop. Add:

```css
@media (pointer: coarse) {
  .btn { padding: 11px 14px; min-height: 44px; }
  .btn.small { padding: 6px 12px; min-height: 44px; }  /* if Clip2Pod has .btn.small */
  .check { gap: 12px; }
}
```

Mouse layout is untouched — the query never matches a fine pointer.

### 6. Whatever the audit finds — same fixes yt-dlFeed applied

Expect the Clip2Pod audit to surface, and fix as yt-dlFeed did:
- **Unlabeled inputs** — the MetaBar Title/Author/Filename fields, Settings
  fields, dialog search boxes. Wrap in `<label>` or add `aria-label`. (The
  `VoicesDialog` toggle already does `aria-label` — extend the pattern.)
- **Heading hierarchy / landmarks** — sidebar section headers (`INTAKE`,
  `TRANSPORT`, `JUNK`, `TOOLS`), band labels: make them real `<h2>`/`<h3>` with
  the `.label` class, give each band an `aria-labelledby`.
- **Modal title** — already `<h2 class="label">`; link it with `aria-labelledby`
  when moving to native `<dialog>` (see item 2).

The render rail (`QueuePanel.svelte`, `aria-hidden="true"`) is fine as-is —
Clip2Pod conveys progress through the live queue-item **text** ("Rendering
(62%) — …"), which is the right call for its layout. Don't force a
`role="progressbar"`; just make sure the queue status region is a live region.

---

## Port into yt-dlFeed (Clip2Pod is ahead)

### A. Themed scrollbars — ✅ DONE in yt-dlFeed 2026-09-09

Ported verbatim from Clip2Pod's `theme.css` (`scrollbar-width` /
`scrollbar-color` + the `::-webkit-scrollbar-*` block).

### B. Themed `::selection` — ✅ DONE in yt-dlFeed 2026-09-09

`::selection { background: color-mix(in srgb, var(--accent) 30%, transparent); }`

### C. Modal entrance animation — still open in yt-dlFeed

yt-dlFeed's modal is now native `<dialog>` with no entrance motion.
Port Clip2Pod's `modal-in` (6px rise + fade, 160ms
`cubic-bezier(0.16, 1, 0.3, 1)`) and its reduced-motion fade-only fallback,
adapted to `.modal[open]` / `.modal::backdrop`. (Do this in the same pass as
Clip2Pod item 2, so both apps land the native-`<dialog>` + animation together.)

### D. Toast slide-in — still open in yt-dlFeed

Clip2Pod's toast slides in 12px from the right over 180ms (reduced-motion:
fade only). yt-dlFeed's toasts appear instantly. Port the animation.

---

## Spec reconciliation (both DESIGN.md files)

- **Supporting sizes.** yt-dlFeed's `DESIGN.md` now has a *Supporting sizes*
  subsection under Typography documenting the 13 / 12.5 / 12 / 9px steps that
  recur below the 4-role frontmatter ramp. Clip2Pod uses the same sizes and
  should carry the same subsection — otherwise `/impeccable audit`'s detector
  flags them as drift in both apps.
- **Token names.** Both frontmatters use the semantic names (`signal-lavender`,
  not `--accent`). Keep them identical so the two `DESIGN.md` files diff cleanly
  — that comparability is the point of a shared system.
- **The QR `#fff`.** Both apps use one literal `#fff` for the feed-dialog QR
  quiet zone. Both DESIGN.md files should note it as the single intentional
  colour exception.
- **Divergences that are correct.** Record these in each DESIGN.md so they read
  as decisions, not drift:
  - yt-dlFeed is a **scrolling single-column stack** (episode library grows
    unbounded); Clip2Pod is a **fixed four-band desk**.
  - yt-dlFeed's queue uses **labelled badges** per job; Clip2Pod uses the
    **circle disc + render rail**.
  - yt-dlFeed reaches for **native primitives** (`<dialog>`, OS `ask()`);
    Clip2Pod's inline expand-in-place confirm is its own valid pattern.

---

## Managing both apps together — going forward

Three models, cheapest to most robust:

### Model 1 — Independent + periodic reconciliation (status quo)
Keep both repos fully independent. After any design change to either, run
`/impeccable audit` on both and reconcile by hand. Each repo keeps a
`docs/design-reconciliation.md` (this file) tracking known divergences.

- **Pro:** zero restructuring; each app free to diverge in layout.
- **Con:** relies on you remembering to reconcile. It already drifted once —
  that's why this document exists.

### Model 2 — Canonical source + sync script (recommended)
Designate one location as canonical for the **shared files only**:
`theme.css`, `Modal.svelte`, `StatusLamp.svelte`, `Toasts.svelte`,
`StartupPromptDialog.svelte`, `DESIGN.md`, `.impeccable/design.json`. Either
pick one app as canonical, or create a tiny third repo/folder `desk-ui/`.

Add a `scripts/sync-design.sh` to the non-canonical app(s) that copies those
files in, plus a CI check that fails if they've drifted:

```sh
# in the downstream repo
for f in src/lib/theme.css src/lib/components/Modal.svelte \
         src/lib/components/StatusLamp.svelte src/lib/components/Toasts.svelte \
         src/lib/components/StartupPromptDialog.svelte \
         DESIGN.md .impeccable/design.json; do
  cp "../desk-ui/$f" "$f"
done
```

- **Pro:** one edit to the shared layer, propagated deliberately. No build-system
  change.
- **Con:** the copied components must stay genuinely identical — if one app needs
  a component to diverge, it leaves the shared set.

### Model 3 — Monorepo / shared package
Move both apps under one repo (`apps/clip2pod`, `apps/yt-dlfeed`) with a
`packages/desk-ui` workspace holding `theme.css`, the shared Svelte components,
and the design spec. Both apps `import` from it.

- **Pro:** true single source of truth; a fix lands once, tooling enforces it.
- **Con:** one-time restructuring; shared components must be parameterised for
  the small real differences (e.g. `StatusLamp`'s status words: `RENDERING` vs
  `DOWNLOADING`).

**Recommendation:** Model 2 now (it's an afternoon and stops the drift), Model 3
if you find yourself syncing more than a couple of times a month.

---

## Appendix — full list of yt-dlFeed changes this session

Grouped by `/impeccable` command; all committed to working tree only.

| Pass | What changed |
|---|---|
| `harden` | Native `<dialog>`; every input labelled; `h1/h2/h3` + landmark `aria-labelledby`; `role="progressbar"` on job bars; empty `<th>` → visually-hidden "Actions"; theme buttons `aria-pressed`; args-field `aria-invalid` + `aria-describedby`; `StartupPromptDialog.answer()` wrapped `try/finally` |
| `adapt` | All flex rows `flex-wrap: wrap` + input `min-width`; `.table-wrap { overflow: auto }`; `@media (pointer: coarse)` 44px targets |
| `animate` | Removed blanket `* { animation: none }` reduced-motion kill → targeted per-loop rules; indeterminate progress bar is a barber-pole that never reads as complete; `OnAir` lamp holds a steady glow under reduced motion |
| `colorize` | Light `--muted` `#6b6d94` → `#5f6186` |
| `document` | Wrote `DESIGN.md` + `.impeccable/design.json` as a sibling of Clip2Pod's; ident tracking `0.18em` → `0.22em` |
| `distill` | Removed dead `tr.selected td` rule (still live in Clip2Pod — keep it there) |
| `polish` | `prefers-color-scheme: light` fallback; `textarea:focus-visible`; modal title `<span>` → `<h2>`; settings subsections `<h2>` → `<h3>` |
| 2nd `harden` | Toast `aria-live` region + non-colour error prefix; per-job progressbar labels; |
| 2nd `adapt` | `.btn` / `.btn.small` `min-height: 44px` under `pointer: coarse` |

yt-dlFeed audit score: **17/20 → 20/20** (projected; verification was
code-level — no live screenshot pass).
