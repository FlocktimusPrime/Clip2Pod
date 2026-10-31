# Reddit Thread Capture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Toolbar / "Narrate" on a Reddit thread sends the post title, body and top first-level comments to Clip2Pod as prepared text, skipping Readability.

**Architecture:** A self-contained extractor function (`extension/*/reddit.js`) runs in the tab via `chrome.scripting.executeScript`, fetches `/comments/<id>.json` same-origin, filters and formats comments. `background.js` routes Reddit threads to it and POSTs `{url, prepared}`. `capture.rs` accepts an optional `prepared` field, routes it straight to narrate, and emits `article-captured` with `reset_gender: true`; `NarrateTab.svelte` resets the gender picker to Unknown before applying.

**Tech Stack:** Rust (Tauri shell, serde), Svelte 5 / TS, plain-JS MV3 extensions (Chrome service worker, Firefox event page), Node 24 `node:test` + `node:vm` for extractor tests (no new deps).

**Spec:** `docs/superpowers/specs/2026-10-31-reddit-thread-capture-design.md`

## Global Constraints

- Fetch URL: `{location.origin}/comments/{id}.json?sort=top&depth=1&limit=500&raw_json=1`. Never the comment subpath.
- Comment cap default **25**, min 1, max 500, stored in `chrome.storage.sync` key `redditCommentCap`; cap applied *after* filtering.
- Drop comments with fewer than **5 words** after markdown stripping.
- Prepared `title` = `"{post title} from r/{subreddit}"`, `author` = `"u/{OP author}"`.
- Manifests: add `"storage"` permission, `"options_ui": { "page": "options.html" }`, `version` → `"0.4.0"`, both builds. No new host permissions.
- The extractor passed to `executeScript` must be one self-contained function (no closures over outer helpers); cap passed via `args: [cap]`.
- `chrome/reddit.js` and `firefox/reddit.js` are byte-identical.
- Errors reuse existing behavior: `!` badge + `console.error`.
- Narrate keeps bare IPC/event names (`article-captured`); no `rip:` prefix here.
- No new npm or cargo dependencies.

## Review Focus

1. **Underscores inside words** (`u/some_user`, `snake_case`) — emphasis stripping must only remove `_`/`*` wrapping words at word boundaries, never interior ones. Test in Task 2.
2. **Score rounding at unit boundaries** — `999_950` must read `1 million upvotes`, not `1000 thousand upvotes`; negative scores read `-4 upvotes`. Test in Task 2.
3. **Self post whose body is empty after stripping** (title-only post, image-only selftext) — `text` starts directly with `Top comments.`, no empty paragraph, no "Linked article" line. Test in Task 2.
4. **Deleted OP** (`author: "[deleted]"`) — prepared `author` is `""` rather than `u/[deleted]`, so the intro and gender lookup don't treat it as a name. Test in Task 2.
5. **Thread URLs with query/hash and look-alike paths** — `…/comments/abc123/?utm_source=share#c` is a thread with id `abc123`; `/user/x/comments/` and `/r/sub/comments/` (no id) are not. Test in Task 2.

---

### Task 1: App accepts prepared payloads

**Files:**
- Modify: `src-tauri/src/capture.rs` (structs at 22-38, `handle` 67-94, `route_article` 96-110, tests 137-162)

**Interfaces:**
- Produces: `article-captured` event payload `{ title, author, text, url, reset_gender: bool }`; `/capture` accepts `{ url, html?, override_hint?, prepared?: { title, author, text } }`.

- [ ] **Step 1: Write failing tests** in the existing `mod tests`:

```rust
#[test]
fn prepared_payload_needs_no_html() {
    let p: CapturePayload = serde_json::from_str(
        r#"{"url":"https://www.reddit.com/comments/abc","prepared":{"title":"T from r/x","author":"u/op","text":"body"}}"#,
    ).unwrap();
    assert_eq!(p.html, "");
    let a = article_for(&p).unwrap();
    assert_eq!((a.title.as_str(), a.author.as_str(), a.text.as_str()), ("T from r/x", "u/op", "body"));
    assert!(a.reset_gender);
}

#[test]
fn legacy_payload_still_parses() {
    let p: CapturePayload = serde_json::from_str(r#"{"url":"https://e.com/a","html":"<html></html>"}"#).unwrap();
    assert!(p.prepared.is_none());
    assert!(!wants_rip(&p));
}

#[test]
fn prepared_beats_rippable_host() {
    let p: CapturePayload = serde_json::from_str(
        r#"{"url":"https://www.youtube.com/watch?v=x","prepared":{"title":"t","author":"a","text":"b"}}"#,
    ).unwrap();
    assert!(!wants_rip(&p));
    let forced: CapturePayload = serde_json::from_str(
        r#"{"url":"https://e.com/a","override_hint":"video","prepared":{"title":"t","author":"a","text":"b"}}"#,
    ).unwrap();
    assert!(wants_rip(&forced));
}
```

Plus one assertion that the HTML path sets `reset_gender: false`: call `article_for` on a legacy payload with a minimal readable article HTML (a `<title>` and a `<p>` of ≥ a few sentences inside `<article>`) and assert `!a.reset_gender`. If `extract_from_html` rejects the fixture, enlarge the paragraph rather than dropping the assertion.

- [ ] **Step 2: Run, expect compile failure** (`article_for`, `wants_rip`, `prepared` missing)

Run: `cargo test -p clip2pod --lib capture --manifest-path src-tauri/Cargo.toml`

- [ ] **Step 3: Implement**
  - `#[derive(Deserialize)] struct Prepared { title: String, author: String, text: String }`
  - `CapturePayload`: `#[serde(default)] html: String`, add `#[serde(default)] prepared: Option<Prepared>`. Update the file header comment (HTML no longer always sent).
  - `CapturedArticle`: add `reset_gender: bool`.
  - `fn wants_rip(p: &CapturePayload) -> bool`: `Some("video")` → true; `Some("article")` → false; `prepared.is_some()` → false; else `rippable_host`.
  - `fn article_for(p: &CapturePayload) -> Result<CapturedArticle, String>`: prepared → clone fields, `reset_gender: true`; else `extract_from_html`, `reset_gender: false`.
  - `handle` uses `wants_rip`; `route_article` becomes emit of `article_for(payload)?`.

- [ ] **Step 4: Run tests, expect PASS** (same command; all capture tests incl. `only_extensions_and_originless_callers_get_in`)

- [ ] **Step 5: Commit** — `feat(capture): accept prepared article payloads`

---

### Task 2: Reddit extractor (`reddit.js`)

**Files:**
- Create: `extension/chrome/reddit.js`, `extension/firefox/reddit.js` (identical copy)
- Create: `extension/test/reddit.test.mjs`

**Interfaces:**
- Produces (top-level globals in a classic script):
  - `isRedditThread(url: string) -> boolean` — host `reddit.com` or `*.reddit.com`; path matches `/comments/<id>` where `<id>` is `[a-z0-9]+` (case-insensitive), optionally preceded by `/r/<sub>`. Query/hash ignored.
  - `async extractRedditThread(cap: number) -> { url, prepared: {title, author, text} } | { error: string }` — self-contained (all helpers nested inside). Reads `location.href`/`location.origin`/`location.pathname`, calls global `fetch`. Never throws: non-OK response → `{ error: "Reddit returned <status>" }`, unparseable body → `{ error: "Reddit response was not JSON" }`, no id → `{ error: "not a Reddit thread" }`.

- [ ] **Step 1: Write the failing tests** in `extension/test/reddit.test.mjs`.

Loader: read `../chrome/reddit.js`, run with `vm.runInContext` in a context `{ URL, location, fetch, JSON }` where `location` is built from a test URL and `fetch` is a stub that records the requested URL and returns `{ ok, status, text: async () => body }` (the extractor must parse via `response.text()` + `JSON.parse` so non-JSON is caught). Fixture helper `listing(post, comments)` → `[{data:{children:[{kind:"t3",data:post}]}},{data:{children:comments.map(c => c.kind ? c : {kind:"t1",data:c})}}]`, with post defaults `{title:"Big news", subreddit:"news", author:"op_user", selftext:"", is_self:true, domain:"self.news"}` and comment defaults `{author:"alice", body:"this is a real comment here", score:10, stickied:false, distinguished:null}`.

Tests (names + assertions):

- `same-file`: `chrome/reddit.js` and `firefox/reddit.js` contents strictly equal.
- `isRedditThread`: true for `https://www.reddit.com/r/news/comments/abc123/slug/`, `https://old.reddit.com/comments/abc123`, `https://np.reddit.com/r/x/comments/abc123/slug/def456/`, `https://www.reddit.com/r/x/comments/abc123/?utm_source=share#c` (RF5); false for `https://www.reddit.com/r/news/`, `https://www.reddit.com/user/x/comments/`, `https://www.reddit.com/r/sub/comments/`, `https://notreddit.com/comments/abc123`, `https://reddit.com.evil.com/comments/abc123`.
- `fetch URL`: from tab `https://old.reddit.com/r/x/comments/abc123/slug/def456/` the stub saw exactly `https://old.reddit.com/comments/abc123.json?sort=top&depth=1&limit=500&raw_json=1`; result `url` equals the tab href.
- `prepared title/author`: `title === "Big news from r/news"`, `author === "u/op_user"`; with post `author:"[deleted]"` → `author === ""` (RF4).
- `self post layout`: selftext `"Hello **world** there."`, two comments → `text ===`
  `"Hello world there.\n\nTop comments.\n\nComment 1, from u/alice, 10 upvotes.\nthis is a real comment here\n\nComment 2, from u/bob, 1 upvote.\n…"`.
- `link post`: `is_self:false, domain:"nytimes.com"` → text starts `"Linked article on nytimes.com.\n\nTop comments."`.
- `empty self body` (RF3): selftext `""` and selftext `"![img](https://preview.redd.it/x.png)"` → text starts `"Top comments.\n\n"`.
- `filters`: each dropped — body `[deleted]`, body `[removed]`, author `[deleted]`, author `AutoModerator`, `stickied:true`, `distinguished:"moderator"`, body `"https://imgur.com/x"`, body `"![gif](giphy|abc)"`, body `"lol same"` (2 words), `{kind:"more"}` child; one surviving comment is numbered `Comment 1`.
- `cap after filter`: 3 junk + 30 good, cap 25 → exactly 25 `Comment N,` headers, last is `Comment 25,`, and they're the first 25 good ones in input order.
- `no comments`: all junk → text ends `"\n\nNo comments."` and contains no `Top comments.`.
- `score phrases` (via one comment each): `0`→`0 upvotes`, `1`→`1 upvote`, `840`→`840 upvotes`, `999`→`999 upvotes`, `1000`→`1 thousand upvotes`, `1234`→`1.2 thousand upvotes`, `15000`→`15 thousand upvotes`, `999950`→`1 million upvotes` (RF2), `1100000`→`1.1 million upvotes`, `-4`→`-4 upvotes` (RF2); `score_hidden:true` → header exactly `Comment 1, from u/alice.`.
- `markdown stripping` (comment bodies, assert stripped text line):
  - `"see [the docs](https://x.com/d) for more info"` → `"see the docs for more info"`
  - `"read this https://x.com/a?b=c right now please"` → `"read this right now please"`
  - `"> quoted words here\n\nmy reply is longer"` → `"quoted words here\n\nmy reply is longer"`
  - `"# Heading\n*bold* _it_ ~~gone~~ ^super and `code` here"` → `"Heading\nbold it gone super and code here"`
  - fenced ```` ```\nlet x = 1;\n``` plus four more words ```` → keeps `let x = 1;`, no backticks
  - `"| a | b |\n|---|---|\n| one | two | three four five"` → no `|`, no `---` row, words kept
  - `"ping u/some_user about my_var_name today ok"` → unchanged (RF1)
  - `"para one words here\n\n\n\npara two words here"` → single `\n\n` between
- `errors`: stub `ok:false,status:403` → `{error:"Reddit returned 403"}`; body `"<html>"` → `{error:"Reddit response was not JSON"}`.

- [ ] **Step 2: Run, expect FAIL** (file missing)

Run: `node --test extension/test/`

- [ ] **Step 3: Implement `extension/chrome/reddit.js`**, then copy to `extension/firefox/reddit.js`.

Comment filter order per spec §2 (raw checks first, then strip, then empty / `< 5` words via `split(/\s+/).filter(Boolean)`). Markdown strip order matters; apply in this sequence:
1. Remove image/GIF embeds: `![…](…)`, `giphy|…` tokens, any URL containing `preview.redd.it`.
2. `[text](url)` → `text`.
3. Remove bare `https?://\S+`.
4. Fenced code: drop the ```` ``` ```` fence lines, keep inner lines. Inline: drop backticks.
5. Table separator rows (`^\s*\|?[\s:|-]+\|?\s*$` with at least one `-`) dropped; remaining `|` → space.
6. Line-leading `>` (repeated) and `#{1,6}\s` removed.
7. Emphasis: `~~x~~` → `x`; `*`/`**`/`_`/`__` only when wrapping text at word boundaries (`(^|\W)([*_]{1,2})(\S.*?\S|\S)\2(?=\W|$)`); `^` dropped.
8. Collapse spaces per line, trim lines, collapse 2+ blank lines to one `\n\n`, trim.

Score phrase: `n < 1000` → `` `${n} upvote${n === 1 ? "" : "s"}` ``; else pick unit by rounding first: `t = Math.round(n / 100) / 10` (thousands); if `t >= 1000` use millions `Math.round(n / 1e5) / 10`. Format with `String(x)` (drops `.0`).

Text assembly: `[bodyPart, commentsPart].filter(Boolean).join("\n\n")` where bodyPart = stripped selftext (self) or `Linked article on ${domain}.` (link); commentsPart = `"Top comments.\n\n" + entries.join("\n\n")` or `"No comments."`; entry = header line + `\n` + stripped body.

- [ ] **Step 4: Run, expect PASS** — `node --test extension/test/`, all tests green.

- [ ] **Step 5: Commit** — `feat(extension): Reddit thread extractor`

---

### Task 3: Wire extension routing, options page, manifests

**Files:**
- Modify: `extension/chrome/background.js`, `extension/firefox/background.js` (`send`, listeners)
- Modify: `extension/chrome/manifest.json`, `extension/firefox/manifest.json`
- Create: `extension/{chrome,firefox}/options.html`, `extension/{chrome,firefox}/options.js` (identical pairs)

**Interfaces:**
- Consumes: `isRedditThread`, `extractRedditThread` from Task 2; `/capture` `prepared` field from Task 1.
- Load the `chrome-extensions` skill before editing (MV3 `executeScript`/`importScripts`/`options_ui`/`storage` reference).

- [ ] **Step 1: Load `reddit.js`.** Chrome: `importScripts("reddit.js");` at top of `background.js`. Firefox manifest: `"background": { "scripts": ["reddit.js", "background.js"] }`.

- [ ] **Step 2: Routing in `send(tab, overrideHint)`.** Before the HTML path: if `overrideHint !== "video"` and `tab.url` host is reddit:
  - `isRedditThread(tab.url)` → `cap` from `chrome.storage.sync.get({ redditCommentCap: 25 })`; `executeScript({ target:{tabId}, func: extractRedditThread, args:[cap] })`; `result.error` → `console.error("Clip2Pod Reddit capture:", result.error)` + `!` badge, return; else POST `{ url, prepared }` (no `html`, no `override_hint`).
  - Not a thread (any `reddit.com` / `*.reddit.com` host) → `console.error("Clip2Pod: not a Reddit thread", tab.url)` + `!` badge, nothing sent.
  - Factor the existing POST + ✓/! response handling into `post(tab, body)` used by both paths. Rip (`"video"`) path unchanged. Same edits in both `background.js` files.
  - `tab.url` is available under `activeTab` after the user gesture; if it's undefined, fall through to the existing HTML path.

- [ ] **Step 3: Options page.** `options.html`: label "Top-level comments per Reddit thread", `<input type="number" id="cap" min="1" max="500">`, script `options.js`. `options.js`: on load set value from `chrome.storage.sync.get({ redditCommentCap: 25 })`; on `change`, clamp to integer 1–500 (invalid → 25), write back to input and `chrome.storage.sync.set`.

- [ ] **Step 4: Manifests (both).** Add `"storage"` to `permissions`, `"options_ui": { "page": "options.html" }`, `"version": "0.4.0"`.

- [ ] **Step 5: Verify**
  - `node --test extension/test/` still PASS.
  - `node -e "for (const b of ['chrome','firefox']) JSON.parse(require('fs').readFileSync('extension/'+b+'/manifest.json'))"` exits 0.
  - `npx web-ext lint -s extension/firefox` → `errors 0` (warnings acceptable only if they predate this change; compare against `git stash` run if unsure).
  - Load unpacked in Chrome: no errors on `chrome://extensions`; options page opens and persists a value across reopen.

- [ ] **Step 6: Commit** — `feat(extension): route Reddit threads to the extractor, add options page`

---

### Task 4: Frontend gender reset

**Files:**
- Modify: `src/lib/types.ts:103-105`
- Modify: `src/lib/components/NarrateTab.svelte:46-48`

**Interfaces:**
- Consumes: `reset_gender` from Task 1. Existing `setGender(g)` (line 305) — not `changeGender`.

- [ ] **Step 1:** `CapturedArticle` gains `reset_gender?: boolean`.
- [ ] **Step 2:** Listener becomes async: `if (e.payload.reset_gender) await setGender("Unknown");` then `applyArticle(e.payload, e.payload.url)`. The existing debounced author-lookup `$effect` then pre-fills from `u/{OP}` — no other change.
- [ ] **Step 3: Verify** — `npm run check` → 0 errors.
- [ ] **Step 4: Commit** — `feat(narrate): reset gender to Unknown on Reddit captures`

---

### Task 5: Docs + end-to-end check

**Files:**
- Modify: `extension/README.md` (Use + Layout sections)
- Modify: `README.md:48-51` ("Four ways in")

- [ ] **Step 1:** `extension/README.md`: add a "Reddit threads" subsection under Use with the spec §1 routing table, what gets read (title, body, top N first-level comments, AutoMod/deleted/short comments skipped), the options page (cap, default 25), and that the `!` badge also covers non-thread reddit pages and a failed `.json` fetch. Layout: note `reddit.js` and `options.*` are identical across builds and tested by `node --test extension/test/`.
- [ ] **Step 2:** `README.md`: extension clause mentions Reddit threads are read as post + top comments.
- [ ] **Step 3: Manual checklist** — run every item in the spec's "Manual test checklist" in Chrome, then the extension items in Firefox (`npm run tauri dev` for the app). Record any failure as a fix before committing.
- [ ] **Step 4: Commit** — `docs: Reddit thread capture`
