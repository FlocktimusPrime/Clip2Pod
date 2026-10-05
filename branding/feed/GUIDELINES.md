# Clip2Pod — Logo Guidelines

## 1. The logo

**Idea:** the lines of a paragraph bend at the right margin into the RSS signal; the
paragraph's full stop is the feed's dot. Where each line turns, a right-pointing arrow is cut
out of it: that's the moment text becomes feed. Text in, feed out: the whole product in one
glance.

**Versions:** horizontal lockup (primary) · stacked lockup · symbol · wordmark.
The symbol has two drawings — the **master** and a pixel-fitted **small cut** (no last line,
no arrows) for 24 px and below.

**Construction** (256 grid): one stroke (28), one row pitch (56), arcs concentric on the dot
(R 56 / 112), dot Ø 36 — slightly larger than the stroke so it reads at the same weight.
The white (reversed) master is 1 unit thinner to offset irradiation. Small cut: a 16-unit
pixel grid, so at 16 px every stroke is exactly 2 px.

**Arrow joins:** a chevron-shaped gap 12 units wide, measured square to its arms, with its
tip 8 units before each bend. The arms lean back 18 for every 26 up or down, so they
clear the stroke. The tips are deliberately sharp, the only corners in an otherwise round
mark, because the sharpness is what makes them read as arrows. They hold down to about
48 px and are gone by 32 px. That's why the small cut leaves them out: a sub-pixel cut
would only blur the line.

## 2. Clear space

Keep **one dot diameter (Ø)** clear on every side. At the 256-unit master Ø = 36; it
scales with the logo. In the horizontal lockup the symbol–wordmark gap is also one Ø.

## 3. Minimum size

| Version | Screen | Print |
|---|---|---|
| Horizontal lockup | 120 px wide | 30 mm wide |
| Stacked lockup | 64 px wide | 18 mm wide |
| Symbol, master | 32 px | 8 mm |
| Symbol, small cut | 16–24 px (tray, favicon, `.ico` frames) | 4–6 mm |

## 4. Colour

All values are DESIGN.md tokens. The mark is the *signal*, so it carries Signal Lavender —
the only chromatic colour (DESIGN.md's One Signal Rule). The wordmark takes Ink.

| Role | Token | HEX | RGB | CMYK (approx.) |
|---|---|---|---|---|
| Mark on dark | Signal Lavender | `#b5abfc` | 181 171 252 | 28 32 0 1 |
| Mark on light | Signal Lavender (light) | `#5b4fc4` | 91 79 196 | 54 60 0 23 |
| Ground / one-colour dark | Desk Black | `#161826` | 22 24 38 | 42 37 0 85 |
| Wordmark on dark | Ink | `#e9e9ed` | 233 233 237 | 2 2 0 7 |
| Wordmark on light | Ink (light) | `#1f2140` | 31 33 64 | 52 48 0 75 |

CMYK is a naive conversion from RGB. Have it proofed before any print run, and don't
assign a Pantone match until it's checked against a swatch book.

**Approved pairs (WCAG contrast):**
- lavender on Desk Black: 8.5:1
- Desk Black on lavender (the rip cover): 8.5:1
- `#5b4fc4` on white: 6.2:1
- `#5b4fc4` on the light floor `#f2f1fa`: 5.6:1
- black on white, and white on Desk Black

**Never:** `#b5abfc` on white. It's only 2.1:1; use `#5b4fc4` instead.

## 5. Feed covers

Covers are 3000 px, with 1400 px JPGs for directories. They are colour-coded so the two
feeds can be told apart at thumbnail size in a podcast app:

- **Narrate:** Desk Black ground, lavender mark, Ink wordmark, `NARRATE` in Muted Ink.
- **Rip:** the same design inverted. Lavender ground, Desk Black mark and wordmark, and
  `RIP` in `#3d3585` (5.0:1 on lavender).

The mode label is JetBrains Mono Bold in uppercase, tracked 0.22 em (the DESIGN.md
`ident` style).

## 6. Typography

- **Wordmark:** Newsreader SemiBold (opsz 72), tracked −6/2000 em, with the pairs C-l,
  2-P and P-o kerned by hand. The wordmark is supplied as outlines; never retype it.
- **Mode labels:** JetBrains Mono Bold.
- **Licences:** both fonts are SIL OFL 1.1, which allows logo use.

## 7. Don'ts

- Don't stretch, rotate or mirror the mark. The arcs face up and right.
- Don't use the master below 32 px or the small cut above 24 px.
- Don't recolour outside the table above.
- Don't add a second accent colour.
- Don't add gradients, glows or outlines.
- Don't separate the lines from the arcs except at the arrow joins, or remove the full
  stop or the arrows, in the master.
- Don't round off the arrow tips, or add arrows to the small cut.
- Don't set the mark on a busy image. Put it on a Desk Black tile instead.

## 8. Files

Everything lives in `branding/feed/`:

- `svg/` holds the masters and all variants.
- `png/` holds the raster exports.
- `web/` holds the favicon and PWA set.
- `board/` holds the presentation.
- `src/` holds the generator.

See `README.md` for what each file is for.
