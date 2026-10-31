# Reddit thread capture

**Date:** 2026-10-31
**Status:** Approved (design grilled 2026-10-31; implementation plan not yet written)

## Goal

Clicking **Send to Clip2Pod** on a Reddit thread narrates the thread properly:
the post title, the post body, and the top first-level comments, never replies.
Today the extension sends the page HTML through Readability, which mangles
Reddit's comment tree.

Out of scope: copy-to-clipboard or .txt export, pasting a Reddit URL into the
app, fetching Reddit from Rust, and reading nested replies.

## Overview

The extension does all the Reddit work inside the user's tab, then hands the
app finished text through a new optional payload field.

```
reddit thread tab ──executeScript──▶ fetch <thread>.json (same origin, user's session)
                                     filter + format top-level comments
                  ──POST /capture { url, prepared: {title, author, text} }──▶ Clip2Pod
                                     skip Readability → "article-captured" (reset_gender)
                                     → Narrate editor, for review as usual
```

Fetching from the page context means the request is same-origin and carries
the user's cookies. That avoids Reddit's 403/429 blocking of requests that
aren't logged in, and it needs no `reddit.com` host permission: `activeTab`
plus `scripting` already cover it.

## 1. Routing (extension `background.js`, both builds)

A tab is a **Reddit thread** when its host is `reddit.com` or any subdomain
(`www.`, `old.`, `new.`, `np.`) and its path matches `/comments/<id>`
(including `/r/<sub>/comments/<id>/<slug>/<commentId>/`).

| Trigger | Reddit thread | Other reddit.com page | Non-Reddit page |
|---|---|---|---|
| Toolbar click | Reddit mode | `!` badge, nothing sent | unchanged |
| Right-click "Narrate" | Reddit mode | `!` badge, nothing sent | unchanged |
| Right-click "Rip" | unchanged (HTML + `override_hint: "video"`) | unchanged | unchanged |

Mobile `/s/…` share links redirect in the browser before the user clicks, so
the tab URL is already canonical. A comment permalink still yields the
whole thread: always fetch `/comments/<id>.json`, never the comment subpath.

## 2. Fetch and extraction (runs in the tab)

`chrome.scripting.executeScript` serializes `func`, so the extractor must be
**one self-contained function**: no closures over background-script helpers.
Pass the cap in with `args: [cap]`.

Request:

```
GET {location.origin}/comments/{id}.json?sort=top&depth=1&limit=500&raw_json=1
```

- `raw_json=1` returns unescaped text (no `&amp;`).
- `depth=1` keeps replies out of the payload.
- `limit=500` over-fetches so the filters below still leave N comments. There's
  no `more` stub expansion: anything beyond one response is ignored by design.

The response is `[postListing, commentListing]`.

**Post** (`postListing.data.children[0].data`): `title`, `subreddit`,
`author`, `selftext`, `is_self`, `domain`.

**Comments**: keep `children` with `kind === "t1"`, ignoring `kind === "more"`.
Keep the API's Top order. Drop a comment if any of these hold:

1. `body` is `[deleted]` or `[removed]`, or `author` is `[deleted]`.
2. `author === "AutoModerator"`, or `stickied`, or `distinguished === "moderator"`.
3. After markdown stripping (§3), it's empty, meaning it was only a link, image or GIF.
4. After markdown stripping, it has fewer than **5 words**.

Then take the first **N** survivors. N is set in the options (§5) and
defaults to **25**. The cap applies *after* filtering.

## 3. Text format

Markdown → plain text, applied to the post body and every comment:

- `[text](url)` → `text`. Bare URLs are removed.
- Images and GIF embeds (`![…](…)`, `giphy|…`, `preview.redd.it` links) are removed.
- `>` quote markers, `#` headings, `*`/`_`/`~~` emphasis, `^` superscript,
  inline/fenced code markers and table pipes are removed, keeping the words.
- Runs of blank lines collapse to one paragraph break.
- Comments are read in full, with no length cap.

The app's existing *Clean for TTS* and junk-phrase finder still apply
afterwards, as for any article.

**Prepared fields:**

- `title`: `"{post title} from r/{subreddit}"`
- `author`: `"u/{OP author}"`
- `text`:

```
{post body}                          ← self post
Linked article on {domain}.          ← link/image/video post, in place of a body

Top comments.

Comment 1, from u/{author}, {score phrase}.
{comment text}

Comment 2, from u/{author}, {score phrase}.
…
```

The app's spoken intro already reads the title and author, so `text` doesn't
repeat them. Comment numbers count the comments that are read (1…N), not their
position on Reddit.

**Score phrase:** "upvotes", rounded for speech.

- `1` → `1 upvote`; `0`–`999` → `840 upvotes`
- `1,000`–`999,999` → one decimal, trailing `.0` dropped: `1.2 thousand upvotes`, `15 thousand upvotes`
- `≥ 1,000,000` → `1.1 million upvotes`
- `score_hidden: true` → leave the phrase out: `Comment 3, from u/name.`

If no comment survives, still send the post with a `No comments.` line in
place of the comment list.

## 4. App changes

### `src-tauri/src/capture.rs`

```rust
#[derive(Deserialize)]
struct Prepared { title: String, author: String, text: String }

struct CapturePayload {
    url: String,
    #[serde(default)] html: String,          // now optional when `prepared` is sent
    #[serde(default)] override_hint: Option<String>,
    #[serde(default)] prepared: Option<Prepared>,
}

struct CapturedArticle { title, author, text, url, reset_gender: bool }
```

- In `handle`, a payload with `prepared` always routes to **narrate**. It
  doesn't consult `rippable_host`, so a Reddit video thread can't auto-route to
  Rip. An explicit `override_hint: "video"` still wins, but the extension never
  sends `prepared` with it.
- `route_article` uses `prepared` as-is when present, skipping
  `extract_from_html`, and emits `article-captured` with `reset_gender: true`.
  The HTML path emits `reset_gender: false`.
- Add unit tests: deserialize a prepared payload with no `html`; deserialize
  the legacy `{url, html}` payload unchanged; a prepared payload on a URL that
  `rippable_host` would accept still routes to narrate.

### Frontend (`src/lib/types.ts`, `NarrateTab.svelte`)

- `CapturedArticle` gains `reset_gender?: boolean`.
- In the `article-captured` listener, when `reset_gender` is true, call
  `setGender("Unknown")` (*not* `changeGender`, so nothing is remembered)
  **before** `applyArticle`. The existing debounced author-lookup effect then
  runs on `u/{OP}`: a saved gender for that username overrides Unknown, and
  no match leaves it Unknown. Without the reset, the picker would keep the
  previous article's gender, because gender is sticky.
- Gender memory is otherwise unchanged. A manual Male/Female pick on a Reddit
  episode is remembered for that `u/username` under the normal rules.

## 5. Extension options page

- New `options.html` + `options.js` in both `chrome/` and `firefox/`: one
  number input, "Top-level comments per Reddit thread", default 25, min 1,
  max 500.
- Stored in `chrome.storage.sync` under `redditCommentCap`. `background.js`
  reads it (falling back to 25) before calling `executeScript`.
- Manifests: add the `"storage"` permission, add
  `"options_ui": { "page": "options.html" }`, and bump `version` to `0.4.0`
  in both builds. No new host permissions.

## 6. Errors

These all use the existing failure behavior: `!` badge plus a `console.error` in
the background/service-worker console.

- A non-thread reddit.com page with toolbar or Narrate.
- A `.json` fetch that isn't OK (private or quarantined subreddit, rate limit),
  or a response that isn't JSON.
- Clip2Pod isn't running.

## 7. Docs

- `extension/README.md`: describe the Reddit mode, the options page and the
  routing table.
- `README.md` (Narrate → "Four ways in"): mention Reddit threads.

## Manual test checklist

- A self post with more than 25 comments: 25 comments read, Top order,
  AutoMod gone, numbering 1–25.
- A link post: "Linked article on {domain}."
- An image or video post with the toolbar click goes to Narrate; right-click
  Rip rips the video.
- Old Reddit, new Reddit and a comment permalink give identical output.
- A thread with only junk comments: "No comments."
- Change the cap in options to 5: 5 comments.
- Gender: set Male on an article, then capture a Reddit thread from an unknown
  OP: the picker shows Unknown. Pick Female, then capture another thread by the
  same OP: it pre-fills Female.
- Repeat the extension checks in Firefox.
