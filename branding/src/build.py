"""Clip2Pod C2P mark -> SVGs. Geometry measured from the 1536px source render.
Needs fonts/Figtree.ttf (OFL): https://github.com/google/fonts/raw/main/ofl/figtree/Figtree%5Bwght%5D.ttf
Run: python build.py  (writes out/*.svg); render.sh in.svg SIZE out.png rasterizes via headless Edge."""
import math, os
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.transformPen import TransformPen

OUT = 'out'
os.makedirs(OUT, exist_ok=True)

BG, LAV, W1C, W2C, WORD = '#0F1021', '#AF93F0', '#896EE5', '#6C55B1', '#E7E3E9'

AX, CY, RO, RI = 730.0, 639.5, 232.5, 138.5
CX, PX = AX - 149.5, AX + 149.5          # bowl centres
TOP, BOT, ITOP, IBOT = CY - RO, CY + RO, CY - RI, CY + RI
CT, PT = AX - 31, AX + 31                # terminal edges either side of the gap
DIAG = 178                               # C tail: x = y - DIAG ; P mirrors it
STEM_R, STEM_B = PT + 86, 985
WC, WR1, WR2, WW = 940.0, 245.5, 339.5, 42


def f(v): return f'{v:.2f}'.rstrip('0').rstrip('.')


def path(items):
    """items: ('L', x, y, fillet_r) | ('A', x, y, R, sweep). Closed loop; last item must not fillet."""
    n = len(items)
    sx, sy = items[-1][1], items[-1][2]
    d = [f'M{f(sx)} {f(sy)}']
    cur = (sx, sy)
    for i, it in enumerate(items):
        if it[0] == 'A':
            _, x, y, R, sw = it
            d.append(f'A{f(R)} {f(R)} 0 0 {sw} {f(x)} {f(y)}')
            cur = (x, y); continue
        _, x, y, r = it
        if r <= 0:
            d.append(f'L{f(x)} {f(y)}'); cur = (x, y); continue
        nx, ny = items[(i + 1) % n][1], items[(i + 1) % n][2]
        ax_, ay_ = cur[0] - x, cur[1] - y; la = math.hypot(ax_, ay_); ax_, ay_ = ax_ / la, ay_ / la
        bx, by = nx - x, ny - y; lb = math.hypot(bx, by); bx, by = bx / lb, by / lb
        th = math.acos(max(-1, min(1, ax_ * bx + ay_ * by)))
        t = r / math.tan(th / 2)
        t1 = (x + ax_ * t, y + ay_ * t); t2 = (x + bx * t, y + by * t)
        sw = 1 if (-ax_) * by - (-ay_) * bx > 0 else 0
        d.append(f'L{f(t1[0])} {f(t1[1])}A{f(r)} {f(r)} 0 0 {sw} {f(t2[0])} {f(t2[1])}')
        cur = t2
    return ''.join(d) + 'Z'


C_PATH = path([
    ('L', CT, TOP, 10), ('L', CT, ITOP, 10), ('L', CX, ITOP, 0),
    ('A', CX, IBOT, RI, 0),
    ('L', IBOT - DIAG, IBOT, 8), ('L', BOT - DIAG, BOT, 12), ('L', CX, BOT, 0),
    ('A', CX, TOP, RO, 1),
])
py_diag = 2 * AX + DIAG - PT             # where P's tail meets the stem's left edge
P_PATH = path([
    ('A', PX, BOT, RO, 1),
    ('L', STEM_R, BOT, 10), ('L', STEM_R, STEM_B, 24), ('L', PT, STEM_B, 4),
    ('L', PT, py_diag, 12), ('L', 2 * AX + DIAG - IBOT, IBOT, 8), ('L', PX, IBOT, 0),
    ('A', PX, ITOP, RI, 0),
    ('L', PT, ITOP, 10), ('L', PT, TOP, 10), ('L', PX, TOP, 0),
])


def glyph_run(font, wght, text, target_h, box, track=0.0):
    """Outline `text`; scale so ink height == target_h; returns (d, ink bbox) placed with ink top-left at box."""
    ft = instantiateVariableFont(TTFont(font), {'wght': wght})
    gs, cm, hm = ft.getGlyphSet(), ft.getBestCmap(), ft['hmtx']
    bp = BoundsPen(gs); x = 0; offs = []
    for ch in text:
        g = cm[ord(ch)]; offs.append((g, x))
        gs[g].draw(TransformPen(bp, (1, 0, 0, 1, x, 0))); x += hm[g][0] + track
    x0, y0, x1, y1 = bp.bounds
    s = target_h / (y1 - y0)
    pen = SVGPathPen(gs, ntos=lambda v: f(v))
    for g, ox in offs:
        # font y-up -> svg y-down, ink top-left at box
        gs[g].draw(TransformPen(pen, (s, 0, 0, -s, box[0] + (ox - x0) * s, box[1] + y1 * s)))
    return pen.getCommands(), (x1 - x0) * s


FIG = 'fonts/Figtree.ttf'
TWO_H = 243
_, two_w = glyph_run(FIG, 700, '2', TWO_H, (0, 0))
TWO_PATH, _ = glyph_run(FIG, 700, '2', TWO_H, (AX - two_w / 2, 533))


def arc(r, deg):
    a = math.radians(deg)
    x, y = WC + r * math.cos(a), CY - r * math.sin(a)
    return f'M{f(x)} {f(y)}A{f(r)} {f(r)} 0 0 1 {f(x)} {f(2 * CY - y)}'


def mark(color=None, waves=True):
    c = color or LAV
    s = [f'<path fill="{c}" d="{C_PATH}"/>', f'<path fill="{c}" d="{P_PATH}"/>', f'<path fill="{c}" d="{TWO_PATH}"/>']
    if waves:
        s.append(f'<g fill="none" stroke-width="{WW}" stroke-linecap="round">'
                 f'<path stroke="{color or W1C}" d="{arc(WR1, 63)}"/>'
                 f'<path stroke="{color or W2C}" d="{arc(WR2, 64.5)}"/></g>')
    return '\n'.join(s)


# ink bbox of the full mark (letters + waves)
MX0, MX1 = CX - RO, WC + WR2 + WW / 2
MY0 = CY - WR2 * math.sin(math.radians(64.5)) - WW / 2
MY1 = STEM_B
MW, MH = MX1 - MX0, MY1 - MY0


def fit(size, frac, bb=None):
    """transform centring bbox bb (default: whole mark) in a size x size box, width = frac*size."""
    x0, y0, x1, y1 = bb or (MX0, MY0, MX1, MY1)
    s = size * frac / (x1 - x0)
    tx = size / 2 - (x0 + x1) / 2 * s; ty = size / 2 - (y0 + y1) / 2 * s
    return f'translate({f(tx)} {f(ty)}) scale({s:.5f})'


def svg(w, body, h=None):
    h = h or w
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {f(w)} {f(h)}" width="{f(w)}" height="{f(h)}">\n{body}\n</svg>\n'


def write(name, s): open(os.path.join(OUT, name), 'w', encoding='utf-8').write(s)


# --- wordmark, placed as in the source lockup ---
WORD_BOX = (274, 1048, 1300, 1274)
_, ww0 = glyph_run(FIG, 600, 'Clip2Pod', WORD_BOX[3] - WORD_BOX[1], (0, 0))
_, ww1 = glyph_run(FIG, 600, 'Clip2Pod', WORD_BOX[3] - WORD_BOX[1], (0, 0), track=100)
track = 100 * ((WORD_BOX[2] - WORD_BOX[0]) - ww0) / (ww1 - ww0)
WORD_PATH, wwid = glyph_run(FIG, 600, 'Clip2Pod', WORD_BOX[3] - WORD_BOX[1],
                            ((WORD_BOX[0] + WORD_BOX[2]) / 2 - (WORD_BOX[2] - WORD_BOX[0]) / 2, WORD_BOX[1]), track)

# --- badges (rip = play, narrate = mic): concentric in the P's counter, occluding the 2's shoulder ---
BADGE = (PX, CY, 100)       # cx, cy, r
GAP = 18


def badge(kind, pos=None, gap=None):
    bx, by, r = pos or BADGE
    GAP_ = GAP if gap is None else gap
    out = [f'<circle cx="{bx}" cy="{by}" r="{r + GAP_}" fill="{BG}"/>', f'<circle cx="{bx}" cy="{by}" r="{r}" fill="{WORD}"/>']
    if kind == 'play':
        h = r * 0.82; w = h * 0.866; ox = bx + w * 0.12
        pts = [(ox - w / 2, by - h / 2), (ox + w / 2, by), (ox - w / 2, by + h / 2)]
        out.append(f'<path fill="{BG}" stroke="{BG}" stroke-width="{f(r * 0.14)}" stroke-linejoin="round" '
                   f'd="M{f(pts[0][0])} {f(pts[0][1])}L{f(pts[1][0])} {f(pts[1][1])}L{f(pts[2][0])} {f(pts[2][1])}Z"/>')
    else:
        u = r / 96  # designed at r=96
        cw, ch = 48 * u, 74 * u
        top = by - 58 * u
        sw = 14 * u
        out.append(f'<rect x="{f(bx - cw / 2)}" y="{f(top)}" width="{f(cw)}" height="{f(ch)}" rx="{f(cw / 2)}" fill="{BG}"/>')
        ur = 40 * u; uy = top + ch - cw / 2 - 8 * u
        out.append(f'<g fill="none" stroke="{BG}" stroke-width="{f(sw)}" stroke-linecap="round">'
                   f'<path d="M{f(bx - ur)} {f(uy)}A{f(ur)} {f(ur)} 0 0 0 {f(bx + ur)} {f(uy)}"/>'
                   f'<path d="M{f(bx)} {f(uy + ur)}V{f(by + 60 * u)}M{f(bx - 24 * u)} {f(by + 60 * u)}H{f(bx + 24 * u)}"/></g>')
    return '\n'.join(out)


def lockup(extra=''):
    return svg(1536, f'<rect width="1536" height="1536" fill="{BG}"/>\n{mark()}\n{extra}\n<path fill="{WORD}" d="{WORD_PATH}"/>')


# icon-only: mark fills the square
write('c2p-mark.svg', svg(1024, f'<g transform="{fit(1024, 0.94)}">{mark()}</g>'))
write('c2p-app-icon.svg', svg(1024, f'<rect width="1024" height="1024" rx="224" fill="{BG}"/>\n<g transform="{fit(1024, 0.84)}">{mark()}</g>'))
write('c2p-mono.svg', svg(1024, f'<g transform="{fit(1024, 0.94)}">{mark("currentColor")}</g>'))
write('c2p-logo.svg', lockup())
for kind, name in (('play', 'rip'), ('mic', 'narrate')):
    write(f'c2p-cover-{name}.svg', svg(3000, f'<rect width="3000" height="3000" fill="{BG}"/>'
                                             f'<g transform="{fit(3000, 0.84)}">{mark()}{badge(kind)}</g>'))
# small sizes (<=32px): waves dropped so the letters, and the 2, get ~25% more pixels
LB = (CX - RO, TOP, PX + RO, STEM_B)
write('c2p-icon-small.svg', svg(1024, f'<rect width="1024" height="1024" rx="224" fill="{BG}"/>'
                                      f'<g transform="{fit(1024, 0.80, LB)}">{mark(waves=False)}</g>'))
write('c2p-mark-small.svg', svg(1024, f'<g transform="{fit(1024, 0.96, LB)}">{mark(waves=False)}</g>'))
