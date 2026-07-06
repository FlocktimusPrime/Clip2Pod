# Send to Clip2Pod — browser extension

Sends the article you are reading (as rendered in your logged-in browser
session, paywalled content included) to the Clip2Pod desktop app.

## Install (Chrome / Brave / Edge)

1. Open `chrome://extensions` (or `brave://extensions`, `edge://extensions`).
2. Enable **Developer mode** (top-right toggle).
3. Click **Load unpacked** and choose this `extension/` folder.

## Use

1. Have Clip2Pod running (window or tray — closing the window keeps it in the tray).
2. Open the article in your browser, then click the **Send to Clip2Pod** toolbar button.
3. Badge shows **✓** — Clip2Pod pops up with the extracted article in the editor.
   Badge **!** means Clip2Pod is not running or the page had no readable article
   (details in the extension's service-worker console).

The extension only acts when clicked, only on the active tab, and only talks
to `127.0.0.1:4737` (the Clip2Pod app on your own machine).
