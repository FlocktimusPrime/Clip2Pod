# Rip with the browser's login (cookies from the extension)

**Date:** 2026-10-31
**Status:** Approved 2026-10-31
**Issue:** #21

## Why

Ripping a Reddit video fails: yt-dlp says `Account authentication is required`.
The user's workaround, `--cookies-from-browser brave` in the Rip arguments, only
works when Brave is fully closed. On Windows, Brave locks its cookie database
while it runs (`ERROR: Could not copy Chrome cookie database`, yt-dlp #7271).
Firefox works but isn't the user's browser. **The user wants to rip with Brave
open.**

The extension already runs inside the browser and can ask it for cookies
directly, with no file lock involved. So the extension sends the clicked site's
cookies along with the rip request, and the app hands them to yt-dlp for that
one run.

## Goals

- Rip from the extension works for any site the user is logged into in the
  browser they clicked from: Brave, Chrome or Firefox, with the browser open.
- The extension asks once per site for permission to read that site's cookies.
- Toolbar clicks that route to Rip use cookies too, as well as right-click Rip.
- The user's saved Rip arguments are never modified.

## Non-goals

- Reading Brave's locked cookie file from the app.
- A cookie or browser picker in Rip settings.
- Cookies for Narrate. Narrate already runs in the user's tab.
- Cookies for rips that don't come from the extension, such as a URL pasted
  into the app. Those behave as today, using whatever the saved arguments say.

## Anti-goals (failure even if it works)

- Standing access to every site's cookies (`<all_urls>` granted at install).
- Cookies written anywhere that outlives the yt-dlp run, or sent to the UI,
  logs, config or any queue event.
- A permission prompt when the user narrates an ordinary article.

## Decided

### Permission: ask once per site

- Manifests add the `"cookies"` permission and an optional all-hosts entry;
  nothing is granted at install. Chrome: `"optional_host_permissions":
  ["*://*/*"]`. Firefox: `"optional_permissions": ["*://*/*"]`.
  *(My call: Firefox only understands `optional_host_permissions` from version
  128, but accepts hosts in `optional_permissions` from earlier versions, so
  `strict_min_version` stays at `115.0`.)*
- On **right-click Rip**, before any `await` (so the click still counts as a
  user gesture), the extension calls
  `chrome.permissions.request({ origins: [<site pattern>] })`. Brave shows
  "Allow … to read and change your data on reddit.com?" the first time; after
  that it's already granted and no prompt shows.
  - Declined, or the request fails → the rip proceeds **without** cookies,
    exactly as today.
- **Site pattern** = `*://*.<base>/*`, where `<base>` is the tab host's
  registrable domain. It must give the same pattern for every subdomain:
  - `www.reddit.com`, `old.reddit.com` → `reddit.com`
  - `www.youtube.com`, `m.youtube.com` → `youtube.com`
  - `www.bbc.co.uk` → `bbc.co.uk`
- **Toolbar click:** the extension can't know in advance whether the app will
  route the page to Rip, and prompting on every toolbar click would break the
  "no prompt when narrating" anti-goal. So a toolbar click **never prompts**.
  It attaches cookies only when the site is already granted, normally from an
  earlier right-click Rip. When the app routes the payload to Narrate, it
  discards the cookies unused.

### What gets sent

- Cookies = `chrome.cookies.getAll({ url: tab.url, storeId })`, exactly the
  cookies the browser would send to that page, which already includes
  parent-domain cookies such as `.reddit.com`. *(My call: no domain-wide sweep,
  so `mail.google.com` cookies never ride along with a `www.google.com` rip.)*
- `storeId` is the tab's cookie store: `tab.cookieStoreId` on Firefox
  (containers, private windows), and on Chrome the store from
  `chrome.cookies.getAllCookieStores()` whose `tabIds` contains the tab.
  *(My call: otherwise a container or private tab would send the wrong
  account's cookies.)*
- Hosts with no registrable domain (IP addresses, `localhost`, single-label
  hosts) skip the permission request and send no cookies.
- Each cookie travels as `{ domain, hostOnly, path, secure, expirationDate?,
  name, value }`, in a new optional field `cookies` on the existing
  `POST /capture` payload.
- Cookies are sent only on Rip paths: right-click Rip, and toolbar clicks
  (see above). Right-click Narrate and the Reddit-thread path never send them.

### App: capture → queue → worker

- `CapturePayload` gains `#[serde(default)] cookies: Option<Vec<Cookie>>`.
  `handle` passes it to `rip_commands::do_enqueue` only when the payload routes
  to Rip; the Narrate path drops it.
- `Job` (in `clip2pod-rip`) holds the cookies in a field marked
  `#[serde(skip)]`, so they never reach the frontend through the
  `rip:queue`/log events. The queue is in-memory only, so cookies are never
  persisted.
- `Job` derives `Debug` and `Clone`, so the cookie field gets a redacting
  `Debug` (prints `<n cookies>`, never values). The worker takes the cookies
  out of the job (`Option::take`) when it starts the run, so finished jobs hold
  none in memory.
- `Queue::enqueue` gains the cookies parameter. Duplicate enqueue rejection is
  unchanged.
- **Worker, for a job with cookies:**
  1. Write the cookies to a temp file in Netscape `cookies.txt` format, in the
     OS temp dir, with a random name.
  2. Build args with the user's template minus any cookie flags, plus
     `--cookies <file>`.
  3. Run yt-dlp.
  4. Delete the file when the run ends: success, failure or panic. A drop
     guard does the deleting.
- **Extension cookies win:** for that run only, strip from the split template
  `--cookies X`, `--cookies=X`, `--cookies-from-browser X`,
  `--cookies-from-browser=X` and `--no-cookies-from-browser`. Saved config is
  untouched, and jobs without cookies use the template as-is.
- The pure pieces live in `clip2pod-rip` (GPL-3.0-only) with unit tests:
  Netscape formatting, cookie-flag stripping, and `build_args` gaining the
  cookie file. Netscape lines:
  - format: `domain \t includeSubdomains \t path \t secure \t expiry \t name \t value`
  - `includeSubdomains` is `TRUE` when `!hostOnly`, and the domain then keeps
    its leading `.`
  - expiry is `expirationDate` truncated to an integer, or `0` for session
    cookies
  - the file must stay writable, because yt-dlp rewrites it on exit
  - a value containing a tab or newline is skipped
  - the file starts with the `# Netscape HTTP Cookie File` header line

### Privacy and docs

- READMEs (Privacy section, extension README): on Rip, the extension sends that
  site's cookies to the app on `127.0.0.1`. They're used for that one download
  and deleted. Permission is per site, revocable in the browser's extension
  settings.
- Extension version bumps to `0.5.0`.
- Issue #21 is closed by the PR. Its body covered the workaround; this
  supersedes it.

## Left to the builder

- How `<base>` is derived. A small built-in list of two-part public suffixes
  (`co.uk`, `com.au`, …) is fine; a full public-suffix list is not wanted. It
  must pass the examples above.
- Where the temp file lives and its exact name. Restrict permissions where the
  OS allows (owner-only on Unix).
- Whether `do_enqueue` takes `Option<Vec<Cookie>>` directly or a small struct.
- Test layout, following the existing patterns: Rust unit tests in the crate,
  Node `node:test` for any pure extension helper (the base-domain function), and
  manual checks for the permission prompt.

## Manual test checklist

- With Brave open, right-click Rip a Reddit video. The first time, a
  permission prompt appears; allow it. The rip succeeds.
- Rip a second Reddit video: no prompt, the rip succeeds.
- Decline the prompt on another site: the rip runs as before, without cookies.
- With `--cookies-from-browser brave` still in the Rip arguments and Brave
  open, an extension rip succeeds. A URL pasted into the app still uses the
  saved flag.
- Toolbar click on a YouTube video on an already-granted site: it rips with
  cookies and shows no prompt. This one is best-effort: yt-dlp warns that
  YouTube rotates cookies taken from a live session, so a failure there isn't
  proof the feature is broken; Reddit is the reference site. Toolbar click on a news article: it narrates,
  with no prompt.
- After a rip, success or failure, no cookies file remains in the temp dir.
- Repeat in Firefox.
