"""Rasterise the SVG masters in ../svg into ../png via headless Edge (transparent background).

Sizes follow the small-size rule in ../GUIDELINES.md: 16 px uses the *-small drawing,
24 px the *-small24 one, 32 px and up the master.
Run after build.py:  python render.py
"""
import os, subprocess, tempfile, pathlib
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
SVG, PNG = HERE.parent / 'svg', HERE.parent / 'png'
PNG.mkdir(exist_ok=True)
EDGE = r'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'


def render(svg, out, w, h=None):
    h = h or w
    with tempfile.TemporaryDirectory() as tmp:
        page = pathlib.Path(tmp) / 'r.html'
        page.write_text(f'<html><body style="margin:0;background:transparent">'
                        f'<img src="{(SVG / svg).as_uri()}" style="display:block;width:{w}px;height:{h}px">'
                        f'</body></html>')
        subprocess.run([EDGE, '--headless=new', '--disable-gpu', '--hide-scrollbars',
                        '--default-background-color=00000000', '--force-device-scale-factor=1',
                        f'--window-size={w},{h}', f'--screenshot={PNG / out}', page.as_uri()],
                       check=True, capture_output=True)


def aspect(svg):
    import re
    vb = re.search(r'viewBox="([^"]+)"', (SVG / svg).read_text()).group(1).split()
    return float(vb[2]) / float(vb[3])


def cut(base, s, suffix=''):
    """The drawing for size s: base-small (16), base-small24 (24) or the master."""
    return {16: f'{base}-small', 24: f'{base}-small24'}.get(s, base) + suffix + '.svg'


if __name__ == '__main__':
    for s in (16, 24, 32, 48, 64, 128, 256, 512, 1024):
        render(cut('feed-symbol', s), f'feed-symbol-{s}.png', s)
        render(cut('feed-app-icon', s), f'feed-icon-{s}.png', s)
    for s in (16, 32, 48, 128):                      # extension icons: no tile, light-ground lavender for the toolbar
        render(cut('feed-symbol', s, '-light'), f'feed-ext-{s}.png', s)
    for s in (16, 24, 32, 64):                       # tray glyphs: lavender (dark taskbar) and white
        render(cut('feed-symbol', s), f'feed-tray-{s}.png', s)
        render(cut('feed-symbol', s, '-white'), f'feed-tray-white-{s}.png', s)
    # the shipped tray asset: the 16 px cut at 2x, which the OS halves cleanly to 16
    render('feed-symbol-small.svg', 'feed-tray-app-32.png', 32)
    for name in ('feed-horizontal', 'feed-horizontal-light', 'feed-stacked', 'feed-stacked-light'):
        w = 1200
        render(name + '.svg', name + '-1200.png', w, round(w / aspect(name + '.svg')))
    for mode in ('narrate', 'rip'):                  # podcast covers: 3000 PNG + JPGs (Apple wants 1400–3000)
        render(f'feed-cover-{mode}.svg', f'feed-cover-{mode}-3000.png', 3000)
        im = Image.open(PNG / f'feed-cover-{mode}-3000.png').convert('RGB')
        im.save(PNG / f'feed-cover-{mode}-3000.jpg', quality=92)
        im.resize((1400, 1400), Image.LANCZOS).save(PNG / f'feed-cover-{mode}-1400.jpg', quality=92)
    print('wrote', len(list(PNG.iterdir())), 'files to', PNG)
