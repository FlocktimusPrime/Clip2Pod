# Send to Clip2Pod — browser extension

Sends the page you are reading (as rendered in your logged-in browser session,
paywalled content included) to the Clip2Pod desktop app
(`POST http://127.0.0.1:4737/capture`).

Clip2Pod has two modes and the extension can reach either:

- **Narrate** — the page's article text is extracted and read aloud by a neural
  voice.
- **Rip** — the page's audio/video track is downloaded (via `yt-dlp`) as an MP3.

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
2. On the page you want:
   - **Click the toolbar button** — Clip2Pod decides: known video hosts
     (YouTube, Vimeo, …) go to **Rip**, everything else to **Narrate**.
   - **Right-click → "Narrate this page with Clip2Pod"** or
     **"Rip this page's audio with Clip2Pod"** — forces the mode. Use this for a
     YouTube page you want *read* (its transcript/description) rather than
     ripped, or a rippable site the auto-router doesn't know.
3. Badge shows **✓** — Clip2Pod pops up on the matching tab. Badge **!** means
   Clip2Pod is not running, or (Narrate) the page had no readable article
   (details in the extension's background/service-worker console).

The extension only acts when you click, only on the active tab, and only talks
to `127.0.0.1:4737` (the Clip2Pod app on your own machine).

### Reddit threads

On a Reddit thread (`reddit.com` or any subdomain, path `/comments/<id>`),
Narrate sends the post and its top comments instead of the page HTML. Replies
are never read.

| Trigger | Reddit thread | Other reddit.com page | Other page |
|---|---|---|---|
| Toolbar click | Reddit mode | `!` badge, nothing sent | unchanged |
| Right-click **Narrate** | Reddit mode | `!` badge, nothing sent | unchanged |
| Right-click **Rip** | unchanged: still rips the video | unchanged | unchanged |

- **Read:** the post title, the body (self posts) or "Linked article on
  {domain}." (link, image and video posts), then the first N top-level comments
  in Reddit's Top order. Markdown becomes plain text.
- **Skipped:** deleted or removed comments, AutoModerator, stickied or
  moderator-distinguished comments, comments that are only a link, image or GIF,
  and comments under 5 words. N is applied after these are dropped.
- **Options:** open the extension's **Options** entry to set **Top-level
  comments per Reddit thread**: default 25, range 1–500, stored in
  `chrome.storage.sync`.
- **Permissions:** no `reddit.com` host permission. The fetch runs in your tab
  with your logged-in session.
- **Gender:** each Reddit capture resets the narrator's gender to Unknown, then
  pre-fills it from Recognized authors if the OP's `u/name` is remembered.
- **Errors:** the `!` badge and a `console.error` in the background console when
  the page is a non-thread Reddit page, the `.json` fetch fails (private or
  quarantined subreddit, rate limit) or doesn't return JSON, or Clip2Pod isn't
  running.

## Layout

`chrome/` and `firefox/` share the icons, `reddit.js` and the options page.
`reddit.js`, `options.html` and `options.js` are byte-identical in both builds;
`node --test extension/test/*.test.mjs` checks that and the extractor's
behaviour. `background.js` differs only in how it loads `reddit.js` and in the
toolbar icon's light/dark switching. Chrome runs MV3 service workers, Firefox runs MV3
event pages and requires a gecko id.

The toolbar icon has no tile, so it needs a lighter variant (`icon{16,32}-light.png`)
on dark toolbars. Firefox picks it via `theme_icons` in its manifest. Chrome has no
equivalent: its `background.js` keeps a hidden offscreen document (`offscreen.html`,
`offscreen` permission) that watches `prefers-color-scheme` and swaps the icon with
`chrome.action.setIcon`. That follows Chrome's light/dark mode, not custom themes.
