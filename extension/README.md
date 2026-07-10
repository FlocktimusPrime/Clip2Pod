# Send to Clip2Pod — browser extension

Sends the article you are reading (as rendered in your logged-in browser
session, paywalled content included) to the Clip2Pod desktop app
(`POST http://127.0.0.1:4737/capture`).

## Install — Chrome / Brave / Edge / Vivaldi

1. `chrome://extensions` (or `brave://extensions`, `edge://extensions`) →
   enable **Developer mode** (top-right toggle)
2. **Load unpacked** → pick the `extension/chrome/` folder

## Install — Firefox

Temporary (resets on restart):

1. `about:debugging#/runtime/this-firefox`
2. **Load Temporary Add-on…** → pick `extension/firefox/manifest.json`

Permanent installs need the add-on signed (or Firefox ESR/Developer Edition
with `xpinstall.signatures.required=false`); zip the `firefox/` folder
contents to build the XPI.

## Use

1. Have Clip2Pod running (window or tray — closing the window keeps it in the tray).
2. Open the article in your browser, then click the **Send to Clip2Pod** toolbar button.
3. Badge shows **✓** — Clip2Pod pops up with the extracted article in the editor.
   Badge **!** means Clip2Pod is not running or the page had no readable article
   (details in the extension's background/service-worker console).

The extension only acts when clicked, only on the active tab, and only talks
to `127.0.0.1:4737` (the Clip2Pod app on your own machine).

## Layout

`chrome/` and `firefox/` differ only in their manifests (Chrome runs MV3
service workers, Firefox runs MV3 event pages and requires a gecko id);
`background.js` and the icons are identical copies.
