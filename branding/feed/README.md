# Feed mark — logo kit

The paragraph-into-RSS mark. Usage rules: [GUIDELINES.md](GUIDELINES.md). Presentation:
`board/board.html` (slides as PNGs in `board/slides/`).

This is the shipped mark (since 0.8.3). The previous C2P mark stays in `../svg`, `../png`
and `../src` for reference.

## Files

| Path | What |
|---|---|
| `svg/feed-symbol.svg` | Master symbol, lavender, for dark grounds |
| `svg/feed-symbol-light.svg` | Symbol for light grounds (`#5b4fc4`) |
| `svg/feed-symbol-{black,desk,white}.svg` | One-colour symbols (white is the thinned reversed drawing) |
| `svg/feed-symbol-small*.svg` | Small cut for ≤ 24 px (tray, favicon, 16/24 px `.ico` frames) |
| `svg/feed-horizontal*.svg` | Primary lockup: default (dark), `-light`, `-black`, `-desk`, `-white` |
| `svg/feed-stacked*.svg` | Stacked lockup: default, `-light`, `-black`, `-white` |
| `svg/feed-wordmark*.svg` | Wordmark only |
| `svg/feed-app-icon.svg`, `-small.svg` | 1024 rounded tile (Desk Black + lavender); `-small` for ≤ 24 px slots |
| `svg/feed-cover-{narrate,rip}.svg` | 3000 px podcast covers |
| `png/feed-icon-{16…1024}.png` | App tile rasters (16/24 from the small cut) |
| `png/feed-symbol-{16…1024}.png` | Transparent symbol rasters |
| `png/feed-tray-{16,24,32,64}.png`, `feed-tray-white-*.png` | Tray glyphs, lavender and white |
| `png/feed-tray-app-32.png` | The shipped tray asset: small cut at 2x |
| `png/feed-{horizontal,stacked}[-light]-1200.png` | Lockup rasters |
| `png/feed-cover-{narrate,rip}-{3000.png,3000.jpg,1400.jpg}` | Feed covers |
| `web/` | `favicon.ico` (16 small cut + 32/48), adaptive `favicon.svg`, touch/PWA icons, `site.webmanifest`, `head-snippet.html` |

## Rebuild

```sh
cd src
# fonts/ needs the two OFL fonts listed at the top of build.py
python build.py    # svg/
python render.py   # png/ (headless Edge + Pillow)
```

The `web/` set was exported with the logo-design skill's `export_variants.py --web-icons`
from `feed-symbol.svg`. The 16 px frame was then replaced with the small cut, and the
ICO/PNG favicons recoloured to `#5b4fc4` so they read on light browser tabs.

## Where it ships (redo these after a rebuild)

- **App icons:** run `npx tauri icon branding/feed/png/feed-icon-1024.png -o src-tauri/icons`.
  Then patch the small slots with the small-cut tile: the `.ico`'s 16/24 frames
  (`png/feed-icon-{16,24}.png`) and `ios/AppIcon-20x20@1x.png` (flattened on white, like
  tauri's other iOS outputs).
- **Extensions:** copy `png/feed-icon-{16,32,48,128}.png` to
  `extension/{chrome,firefox}/icons/icon*.png`. The README header also uses the
  128 px icon.
- **Tray:** copy `png/feed-tray-app-32.png` (the small cut at 2x) to
  `src-tauri/assets/tray-icon.png`.
- **Webview favicon:** copy `png/feed-icon-32.png` to `static/favicon.png`.
- Covers (`src-tauri/src/feed.rs` embeds both):
  - `png/feed-cover-narrate-3000.png` → `src-tauri/assets/cover.png` (served at `/tts/cover.png`)
  - `png/feed-cover-rip-3000.jpg` → `src-tauri/assets/cover.jpg` (served at `/video/cover.jpg`, unless
    the user has put a `cover.jpg` in the rip output folder)
