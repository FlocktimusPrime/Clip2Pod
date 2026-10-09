"""Clip2Pod "Feed" mark -> SVG masters and variants.

The mark: the lines of a paragraph bend at the right margin into the RSS arcs;
the paragraph's full stop is the feed's dot, and a chevron cut where each line turns
marks the conversion (the small cuts keep it as a 1 px break). Everything is built from one stroke
width, one row pitch and concentric radii on a 256 grid; no strokes or live text
ship in the output (letters are outlined from the fonts below).

Fonts (OFL, not committed — download into fonts/):
  https://github.com/google/fonts/raw/main/ofl/newsreader/Newsreader%5Bopsz,wght%5D.ttf
  https://github.com/google/fonts/raw/main/ofl/jetbrainsmono/JetBrainsMono%5Bwght%5D.ttf
Run: python build.py   (writes ../svg/*.svg; see ../README.md for the PNG step)
"""
import math, os
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.transformPen import TransformPen

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, '..', 'svg')
FONTS = os.path.join(HERE, 'fonts')
os.makedirs(OUT, exist_ok=True)

# DESIGN.md tokens
LAV, LAV_LIGHT = '#b5abfc', '#5b4fc4'     # Signal Lavender (dark / light theme)
DESK, INK, INK_LIGHT = '#161826', '#e9e9ed', '#1f2140'
MUTED = '#9b9bb8'


def f(v):
    return f'{v:.2f}'.rstrip('0').rstrip('.')


# ---------------------------------------------------------------- geometry

def capsule(x0, y, x1, w):
    r = w / 2
    return (f'M{f(x0)} {f(y - r)}H{f(x1)}A{f(r)} {f(r)} 0 0 1 {f(x1)} {f(y + r)}'
            f'H{f(x0)}A{f(r)} {f(r)} 0 0 1 {f(x0)} {f(y - r)}Z')


def disc(cx, cy, r):
    return (f'M{f(cx - r)} {f(cy)}A{f(r)} {f(r)} 0 1 1 {f(cx + r)} {f(cy)}'
            f'A{f(r)} {f(r)} 0 1 1 {f(cx - r)} {f(cy)}Z')


def quarter(O, R, w):
    """Quarter ring from 12 o'clock to 3 o'clock round O, round cap at the lower end."""
    r = w / 2
    ro, ri = R + r, R - r
    return (f'M{f(O[0])} {f(O[1] - ro)}A{f(ro)} {f(ro)} 0 0 1 {f(O[0] + ro)} {f(O[1])}'
            f'A{f(r)} {f(r)} 0 0 1 {f(O[0] + ri)} {f(O[1])}'
            f'A{f(ri)} {f(ri)} 0 0 0 {f(O[0])} {f(O[1] - ri)}Z')


def arrow_line(x0, O, R, w, arrow):
    """A line that bends into its arc, cut by a right-pointing chevron gap just before the bend.

    arrow = (tip, run, rise, gap): the gap's centreline is a chevron whose tip sits `tip`
    units left of the bend, with arms that go `run` left for every `rise` up or down; `gap`
    is the cut's width measured square to the arms. Two subpaths: the line ending in a point,
    and a notched stub that runs on into the quarter ring.
    """
    tip, run, rise, gap = arrow
    r = w / 2
    y = O[1] - R
    ro, ri = R + r, R - r
    shift = (gap / 2) / math.sin(math.atan2(rise, run))   # half the gap, measured along x
    lean = r * run / rise                                   # how far the arms lean back over half a stroke
    tl, tr = O[0] - tip - shift, O[0] - tip + shift         # tips of the two cut edges
    left = (f'M{f(x0)} {f(y - r)}H{f(tl - lean)}L{f(tl)} {f(y)}L{f(tl - lean)} {f(y + r)}'
            f'H{f(x0)}A{f(r)} {f(r)} 0 0 1 {f(x0)} {f(y - r)}Z')
    right = (f'M{f(tr - lean)} {f(y - r)}H{f(O[0])}A{f(ro)} {f(ro)} 0 0 1 {f(O[0] + ro)} {f(O[1])}'
             f'A{f(r)} {f(r)} 0 0 1 {f(O[0] + ri)} {f(O[1])}'
             f'A{f(ri)} {f(ri)} 0 0 0 {f(O[0])} {f(O[1] - ri)}'
             f'H{f(tr - lean)}L{f(tr)} {f(y)}Z')
    return left + right


def feed(w=28, pitch=56, dot=18, O=(114, 179), x0=30, last_line=True, stop_gap=16, arrow=None):
    """All subpaths wind clockwise, so overlaps union under the default nonzero rule."""
    d = ''
    for k in (2, 1):                          # the two lines that bend into arcs
        R = k * pitch
        d += (arrow_line(x0, O, R, w, arrow) if arrow
              else capsule(x0, O[1] - R, O[0], w) + quarter(O, R, w))
    if last_line:                             # the short last line, then the full stop
        d += capsule(x0, O[1], O[0] - dot - stop_gap - w / 2, w)
    d += disc(O[0], O[1], dot)
    return d


# the arrow joins: a chevron gap 8 units before each bend, arms 18 run : 26 rise, gap 12
ARROW = (8, 18, 26, 12)
# small cuts: same chevron, gap widened to 16 units (1 px at 16 px) so it survives as a break
ARROW_SMALL = (8, 18, 26, 16)
# master: stroke 28, pitch 56, dot Ø36 (optically equal to the stroke), ink box 16..240 × 53..197
MARK = feed(arrow=ARROW)
# reversed (light on dark): 1 unit thinner to offset irradiation
MARK_REV = feed(w=27, dot=17.5, arrow=ARROW)


def small_cut(size, s, gap, O, x0):
    """The whole mark (all three lines, the dot) fitted to whole pixels at `size` px.

    Measured in pixels: stroke `s`, `gap` between rows, arcs and dot, dot as wide as the
    stroke; scaled back to the 256 grid. The arrow joins are kept, with the gap widened to
    one pixel (16 units): at these sizes they read as a break, not an arrow.
    """
    k = 256 / size
    return feed(w=s * k, pitch=(s + gap) * k, dot=s / 2 * k, O=(O[0] * k, O[1] * k),
                x0=x0 * k, stop_gap=gap * k, arrow=ARROW_SMALL)


# small cuts: 16 px (stroke 2, gap 1; ink 1..15 x 4..12) and 24 px (stroke 3, gap 1;
# ink 2..22 x 6..17). Each lands on whole pixels only at its own size (and its 2x).
MARK_SMALL = small_cut(16, s=2, gap=1, O=(8, 11), x0=2)
MARK_SMALL24 = small_cut(24, s=3, gap=1, O=(12.5, 15.5), x0=3.5)
MARK_BOX = (16, 53, 240, 197)


# ---------------------------------------------------------------- type

_fonts = {}


def font(name, axes):
    key = (name, tuple(sorted(axes.items())))
    if key not in _fonts:
        _fonts[key] = instantiateVariableFont(TTFont(os.path.join(FONTS, name)), axes)
    return _fonts[key]


SERIF = ('Newsreader%5Bopsz,wght%5D.ttf', {'wght': 600, 'opsz': 72})
MONO = ('JetBrainsMono%5Bwght%5D.ttf', {'wght': 700})
# manual pair kerning (font units) — no shaper here, so GPOS kerning is set by eye
KERN = {('2', 'P'): -20, ('P', 'o'): -30, ('C', 'l'): -10}


def outline(spec, text, cap_h, x, baseline, track=0, kern=None):
    """Outline text so its cap height is cap_h; returns (path d, ink bounds)."""
    ft = font(*spec)
    gs, cm, hm = ft.getGlyphSet(), ft.getBestCmap(), ft['hmtx']
    s = cap_h / ft['OS/2'].sCapHeight
    pen, bp = SVGPathPen(gs, ntos=f), BoundsPen(gs)
    adv = 0
    for i, ch in enumerate(text):
        if ch == ' ':
            adv += hm[cm[32]][0] + track
            continue
        g = cm[ord(ch)]
        t = (s, 0, 0, -s, x + adv * s, baseline)
        gs[g].draw(TransformPen(pen, t))
        gs[g].draw(TransformPen(bp, t))
        adv += hm[g][0] + track
        if kern and i + 1 < len(text):
            adv += kern.get((ch, text[i + 1]), 0)
    return pen.getCommands(), bp.bounds


def wordmark(cap_h, x, baseline):
    return outline(SERIF, 'Clip2Pod', cap_h, x, baseline, track=-6, kern=KERN)


# ---------------------------------------------------------------- files

def write(name, vb, body, title='Clip2Pod logo'):
    x, y, w, h = vb
    open(os.path.join(OUT, name + '.svg'), 'w', newline='\n').write(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{f(x)} {f(y)} {f(w)} {f(h)}" '
        f'width="{f(w)}" height="{f(h)}" role="img" aria-labelledby="t"><title id="t">{title}</title>'
        f'{body}</svg>\n')


def p(d, fill):
    return f'<path fill="{fill}" d="{d}"/>'


def g(tx, ty, s, inner):
    return f'<g transform="translate({f(tx)} {f(ty)}) scale({f(s)})">{inner}</g>'


def pad(box, m):
    x0, y0 = math.floor(box[0] - m), math.floor(box[1] - m)   # integer canvas
    x1, y1 = math.ceil(box[2] + m), math.ceil(box[3] + m)
    return (x0, y0, x1 - x0, y1 - y0)


def union(*boxes):
    return (min(b[0] for b in boxes), min(b[1] for b in boxes),
            max(b[2] for b in boxes), max(b[3] for b in boxes))


def symbols():
    sq = (0, 0, 256, 256)
    write('feed-symbol', sq, p(MARK, LAV))                       # master, on dark
    write('feed-symbol-light', sq, p(MARK, LAV_LIGHT))           # on light grounds
    write('feed-symbol-black', sq, p(MARK, '#000000'))
    write('feed-symbol-desk', sq, p(MARK, DESK))
    write('feed-symbol-white', sq, p(MARK_REV, '#ffffff'))       # reversed, thinned
    for name, mark in (('feed-symbol-small', MARK_SMALL), ('feed-symbol-small24', MARK_SMALL24)):
        write(name, sq, p(mark, LAV))                            # 16 px / 24 px (and 2x)
        write(name + '-light', sq, p(mark, LAV_LIGHT))
        write(name + '-black', sq, p(mark, '#000000'))
        write(name + '-white', sq, p(mark, '#ffffff'))


# lockup proportions (symbol at 256 scale): cap height 76, gap = one dot diameter (36)
CAP, GAP = 76, 36


def horizontal():
    cy = (MARK_BOX[1] + MARK_BOX[3]) / 2                         # optical centre of the mark
    d0, b0 = wordmark(CAP, 0, 0)
    dx = MARK_BOX[2] + GAP - b0[0]
    base = cy + CAP / 2
    d, b = wordmark(CAP, dx, base)
    vb = pad(union(MARK_BOX, b), 24)
    for suffix, sym, word, mark in (('', LAV, INK, MARK), ('-light', LAV_LIGHT, INK_LIGHT, MARK),
                                    ('-black', '#000000', '#000000', MARK),
                                    ('-desk', DESK, DESK, MARK),
                                    ('-white', '#ffffff', '#ffffff', MARK_REV)):
        write('feed-horizontal' + suffix, vb, p(mark, sym) + p(d, word))
    return vb


def stacked():
    cap = 58
    d0, b0 = wordmark(cap, 0, 0)
    ww = b0[2] - b0[0]
    mx = (MARK_BOX[0] + MARK_BOX[2]) / 2
    base = MARK_BOX[3] + 44 + cap
    d, b = wordmark(cap, mx - ww / 2 - b0[0], base)
    vb = pad(union(MARK_BOX, b), 24)
    for suffix, sym, word, mark in (('', LAV, INK, MARK), ('-light', LAV_LIGHT, INK_LIGHT, MARK),
                                    ('-black', '#000000', '#000000', MARK),
                                    ('-white', '#ffffff', '#ffffff', MARK_REV)):
        write('feed-stacked' + suffix, vb, p(mark, sym) + p(d, word))


def wordmark_only():
    d, b = wordmark(CAP, 0, 0)
    vb = pad(b, 16)
    for suffix, col in (('', INK), ('-light', INK_LIGHT), ('-black', '#000000'), ('-white', '#ffffff')):
        write('feed-wordmark' + suffix, vb, p(d, col))


def tiles():
    """Rounded-square app tile: Desk Black ground, lavender mark at 62 % of the tile."""
    S = 1024
    rx = round(S * 0.225)
    mw = MARK_BOX[2] - MARK_BOX[0]
    for name, mark in (('feed-app-icon', MARK), ('feed-app-icon-small', MARK_SMALL),
                       ('feed-app-icon-small24', MARK_SMALL24)):
        if mark is not MARK:                                      # 1:1 on the tile so the pixel grid holds
            s, tx, ty = S / 256, 0, 0
        else:
            s = S * 0.62 / mw
            tx = S / 2 - 128 * s
            ty = S / 2 - 125 * s - S * 0.01                       # optical lift
        body = (f'<rect width="{S}" height="{S}" rx="{rx}" fill="{DESK}"/>' + g(tx, ty, s, p(mark, LAV)))
        write(name, (0, 0, S, S), body, 'Clip2Pod app icon')


def cover(mode, label):
    """3000 px podcast cover. Narrate: dark ground, lavender mark. Rip: the same, inverted."""
    S = 3000
    bg, mark_c, word_c, label_c = ((DESK, LAV, INK, MUTED) if mode == 'narrate'
                                   else (LAV, DESK, DESK, '#3d3585'))
    mw = MARK_BOX[2] - MARK_BOX[0]
    ms = 1500 / mw
    mh = (MARK_BOX[3] - MARK_BOX[1]) * ms
    wcap, lcap, g1, g2 = 250, 132, 250, 170
    total = mh + g1 + wcap + g2 + lcap
    top = (S - total) / 2 - 40
    tx = S / 2 - 128 * ms
    ty = top - MARK_BOX[1] * ms
    base_w = top + mh + g1 + wcap
    d0, b0 = wordmark(wcap, 0, 0)
    dw, _ = wordmark(wcap, S / 2 - (b0[2] - b0[0]) / 2 - b0[0], base_w)
    base_l = base_w + g2 + lcap
    tr = 0.22 * font(*MONO)['head'].unitsPerEm                    # DESIGN.md ident tracking, 0.22em
    l0, lb0 = outline(MONO, label, lcap, 0, 0, track=tr)
    dl, _ = outline(MONO, label, lcap, S / 2 - (lb0[2] - lb0[0]) / 2 - lb0[0], base_l, track=tr)
    body = (f'<rect width="{S}" height="{S}" fill="{bg}"/>' + g(tx, ty, ms, p(MARK, mark_c))
            + p(dw, word_c) + p(dl, label_c))
    write(f'feed-cover-{mode}', (0, 0, S, S), body, f'Clip2Pod {label.lower()} feed cover')


if __name__ == '__main__':
    symbols()
    horizontal()
    stacked()
    wordmark_only()
    tiles()
    cover('narrate', 'NARRATE')
    cover('rip', 'RIP')
    print('wrote', len(os.listdir(OUT)), 'files to', os.path.normpath(OUT))
