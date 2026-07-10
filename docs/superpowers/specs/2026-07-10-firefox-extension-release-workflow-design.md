# Firefox extension + cross-platform release workflow

**Date:** 2026-07-10
**Status:** Approved

## Goal

Add a Firefox variant of the "Send to Clip2Pod" browser extension and a GitHub
Actions release workflow that builds installers for Linux, macOS, and Windows.
Pattern mirrors the sibling project YT-DLFeed, which already ships both.

## 1. Firefox extension

Restructure `extension/` into per-browser folders:

```
extension/
├── README.md          # covers both browsers
├── chrome/            # existing files, moved as-is
│   ├── manifest.json  # MV3, background.service_worker
│   ├── background.js
│   └── icons/
└── firefox/           # new
    ├── manifest.json  # MV3, background.scripts (event page)
    ├── background.js  # identical copy of chrome/background.js
    └── icons/         # identical copies
```

Firefox manifest differences from Chrome:

- `"background": { "scripts": ["background.js"] }` — Firefox runs MV3 event
  pages, not service workers.
- `"browser_specific_settings": { "gecko": { "id": "clip2pod@kketover",
  "strict_min_version": "115.0" } }` — required for any Firefox install.
- Everything else identical: `activeTab` + `scripting` permissions,
  `http://127.0.0.1:4737/*` host permission, same icons and action.

`background.js` is unchanged — the `chrome.*` namespace works in Firefox, and
the capture server already sends permissive CORS headers
(`src-tauri/src/capture.rs`), so the localhost fetch succeeds.

Distribution is unsigned (same as YT-DLFeed): temporary install via
`about:debugging#/runtime/this-firefox`; permanent installs need Firefox
ESR/Developer Edition with `xpinstall.signatures.required=false`, zipping the
`firefox/` folder contents into an XPI.

Docs updated: `extension/README.md` rewritten to cover both browsers; main
`README.md` paths updated from `extension/` to `extension/chrome/` (logo image
path, install instructions, repo-layout table).

## 2. Release workflow

`.github/workflows/release.yml`, near-copy of YT-DLFeed's:

- **Trigger:** push of tags matching `v*`.
- **Matrix:** `macos-latest` (aarch64 + x86_64 targets), `ubuntu-22.04`
  (AppImage/deb/rpm), `windows-latest` (msi/nsis).
- **Ubuntu deps:** `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
  patchelf xdg-utils` (appindicator needed for the tray).
- **Test gate (ubuntu only):** `cargo test -p clip2pod-core` before bundling.
- **Bundling:** `tauri-apps/tauri-action@v1`, creates a **draft** release with
  all platform installers attached. macOS/Windows builds are unsigned.
- Rust cache via `swatinem/rust-cache@v2`, Node LTS with npm cache.

## Out of scope (YAGNI)

- AMO signing / listed publication — no Mozilla account secrets wanted.
- Extension zips as release assets — users install from the repo folder.
- `NO_STRIP` for AppImage — a CachyOS-local linuxdeploy issue; ubuntu CI
  runners are unaffected.
- macOS/Windows code signing — no certificates.
