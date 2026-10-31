# Rip With Browser Cookies Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rips sent from the extension carry the clicked site's cookies from the browser, so yt-dlp can download login-gated media (Reddit) while Brave stays open.

**Architecture:** The extension asks once per site for host permission (right-click Rip), reads the tab's cookies with `chrome.cookies.getAll({url, storeId})`, and adds `cookies` to the `/capture` payload. `capture.rs` hands them to the rip queue; `Job` keeps them out of serialization and `Debug`; `Queue::start_next` moves them into the job the worker runs; the worker writes a temp Netscape cookies file (deleted on drop), strips cookie flags from the saved template for that run, and appends `--cookies <file>`.

**Tech Stack:** Rust (`clip2pod-rip` crate, GPL-3.0-only; Tauri shell), plain-JS MV3 extension (Chrome + Firefox builds), Node `node:test` + `node:vm`.

**Spec:** `docs/superpowers/specs/2026-10-31-rip-browser-cookies-design.md`

## Global Constraints

- Permission is per site, on right-click Rip only: `chrome.permissions.request({ origins: ["*://*.<base>/*"] })`, called before any `await` in the click handler. A toolbar click never prompts; it attaches cookies only if `chrome.permissions.contains` is already true.
- Manifests: add `"cookies"` permission. Chrome `"optional_host_permissions": ["*://*/*"]`; Firefox `"optional_permissions": ["*://*/*"]`; Firefox `strict_min_version` stays `"115.0"`. Version `0.5.0` both builds. No change to `host_permissions`.
- Cookies sent = `chrome.cookies.getAll({ url: tab.url, storeId })`, each as `{ domain, hostOnly, path, secure, expirationDate?, name, value }`, in optional payload field `cookies`. Never on Narrate or the Reddit-thread path.
- Declined / failed permission, or any cookie read error → rip proceeds without cookies.
- Cookies never in serialized `Job`, `Debug` output, logs, config or queue events; queue stays in-memory.
- Extension cookies win for that run: strip `--cookies X`, `--cookies=X`, `--cookies-from-browser X`, `--cookies-from-browser=X`, `--no-cookies`, `--no-cookies-from-browser` from the split template, then append `--cookies <file>`. Saved config untouched; jobs without cookies use the template as-is.
- Netscape file: first line `# Netscape HTTP Cookie File`; lines `domain\tincludeSubdomains\tpath\tsecure\texpiry\tname\tvalue`; `includeSubdomains` `TRUE` iff `!hostOnly` (domain keeps its leading `.`); expiry = `expirationDate` truncated to integer, `0` if absent; cookies whose value contains `\t`/`\n`/`\r` skipped; file writable; deleted when the run ends.
- `clip2pod-rip` stays GPL-3.0-only; no new dependencies in any crate or npm.
- Rip naming: `rip_*` commands, `rip:*` events unchanged.
- Extension tests: `node --test extension/test/*.test.mjs`. Shared extension files byte-identical across builds.

## Review Focus

1. **Saved template already has `--cookies-from-browser brave`** (the user's current setup) — an extension rip must not pass both flags; test in Task 1 (`strip_cookie_flags` cases incl. `=` forms and a following value).
2. **yt-dlp fails or is cancelled mid-run** — cookie file still deleted; Task 1 tests `CookieFile` drop removes the file; Task 3 keeps the guard alive for the whole `run_job` scope.
3. **Debug-printing a job** (panic messages, `dbg!`) — no cookie value appears; Task 2 test.
4. **Permission declined / `getAll` throws** — rip still sent, no `cookies` key; Task 4 test via stubbed `chrome`.
5. **Hosts with no registrable domain** (`127.0.0.1`, `localhost`, `[::1]`, `intranet`) — no permission request, no cookies, no exception; Task 4 test.

---

### Task 1: Cookie file + arg building (`clip2pod-rip`)

**Files:**
- Create: `src-tauri/crates/clip2pod-rip/src/cookies.rs`
- Modify: `src-tauri/crates/clip2pod-rip/src/lib.rs` (add `pub mod cookies;`)
- Modify: `src-tauri/crates/clip2pod-rip/src/ytdlp.rs:65-77` (`build_args`) and its tests
- Modify: `src-tauri/src/rip_worker.rs:107` (pass `None` for now)

**Interfaces:**
- Produces:
  - `#[derive(Clone, Deserialize)] #[serde(rename_all = "camelCase")] pub struct Cookie { pub domain: String, pub host_only: bool, pub path: String, pub secure: bool, #[serde(default)] pub expiration_date: Option<f64>, pub name: String, pub value: String }` with a manual `Debug` that prints domain + name only.
  - `pub fn netscape(cookies: &[Cookie]) -> String`
  - `pub fn strip_cookie_flags(args: Vec<String>) -> Vec<String>`
  - `pub struct CookieFile` — `pub fn write(cookies: &[Cookie]) -> std::io::Result<CookieFile>`, `pub fn path(&self) -> &Path`, `Drop` removes the file (ignore errors). File in `std::env::temp_dir()` named `clip2pod-cookies-<uuid v4>.txt`; on Unix created with mode `0o600` (`OpenOptionsExt`).
  - `pub fn build_args(template: &str, output_dir: &Path, url: &str, cookie_file: Option<&Path>) -> Result<Vec<String>, String>` — when `Some`, applies `strip_cookie_flags` to the split template and appends `--cookies <path>` before `--newline`.

- [ ] **Step 1: Write failing tests** in `cookies.rs` `mod tests` and `ytdlp.rs` tests:
  - `netscape_formats_host_only_and_domain_cookies`: `{domain:"www.reddit.com",host_only:true,path:"/",secure:true,expiration_date:Some(1767225600.7),name:"a",value:"1"}` → line `www.reddit.com\tFALSE\t/\tTRUE\t1767225600\ta\t1`; `{domain:".reddit.com",host_only:false,…,expiration_date:None,name:"reddit_session",value:"x"}` → `.reddit.com\tTRUE\t/\tTRUE\t0\treddit_session\tx`; output starts with `# Netscape HTTP Cookie File\n`.
  - `netscape_skips_values_with_tabs_or_newlines`: values `"a\tb"`, `"a\nb"`, `"a\rb"` absent; sibling valid cookie present.
  - `strip_cookie_flags_removes_every_form` (RF1): input `["-x","--cookies","/c.txt","--cookies-from-browser","brave","--cookies=/d.txt","--cookies-from-browser=firefox","--no-cookies","--no-cookies-from-browser","-o","%(title)s.%(ext)s"]` → `["-x","-o","%(title)s.%(ext)s"]`; `["--cookies"]` (trailing, no value) → `[]`.
  - `cookie_file_written_then_deleted_on_drop` (RF2): `CookieFile::write` → path exists, contents == `netscape(..)`; after `drop` → path gone.
  - `build_args_with_cookie_file_replaces_saved_cookie_flags`: template `"--extract-audio --cookies-from-browser brave"`, `Some(Path::new("/t/c.txt"))` → contains `["--cookies","/t/c.txt"]` once, no `--cookies-from-browser`, still ends `["--", url]`.
  - existing `build_args` tests pass `None` and keep their assertions; template with `--cookies-from-browser brave` and `None` keeps it.
  - `cookie_debug_hides_value`: `format!("{:?}", cookie)` doesn't contain the value.
- [ ] **Step 2: Run, expect FAIL** — `cargo test -p clip2pod-rip --manifest-path src-tauri/Cargo.toml` (compile errors).
- [ ] **Step 3: Implement** per Interfaces; update the one shell caller in `rip_worker.rs` to `build_args(&template, &dir, &job.url, None)`.
- [ ] **Step 4: Run, expect PASS** — same command, plus `cargo check --workspace --manifest-path src-tauri/Cargo.toml`.
- [ ] **Step 5: Commit** — `feat(rip): cookie file and cookie-flag override for yt-dlp runs`

---

### Task 2: Jobs carry cookies, redacted (`clip2pod-rip` queue)

**Files:**
- Modify: `src-tauri/crates/clip2pod-rip/src/queue.rs` (`Job` 24-37, `enqueue` 48, `start_next` 72-77, tests)
- Modify: callers of `Queue::enqueue` (`src-tauri/src/rip_commands.rs:27`) — pass `None` for now

**Interfaces:**
- Consumes: `cookies::Cookie` (Task 1).
- Produces:
  - `#[derive(Clone, Default)] pub struct CookieJar(pub Vec<Cookie>)` in `cookies.rs`, manual `Debug` → `CookieJar(<n cookies>)`.
  - `Job` gains `#[serde(skip)] pub cookies: Option<CookieJar>`.
  - `pub fn enqueue(&mut self, url: &str, cookies: Option<CookieJar>) -> Result<String, String>` (duplicate rule unchanged).
  - `start_next` moves the stored job's cookies into the returned clone (`take()`), so the queue never holds them for a running/finished job.

- [ ] **Step 1: Write failing tests** in `queue.rs`:
  - `cookies_are_not_serialized_or_debug_printed` (RF3): enqueue with a jar holding value `"SECRET"`; `serde_json::to_string(&q.jobs()[0])` and `format!("{:?}", q.jobs()[0])` don't contain `SECRET`; Debug contains `1 cookies`.
  - `start_next_hands_cookies_to_the_runner_once`: `start_next()` returns `Some(job)` with `job.cookies` set; afterwards `q.jobs()[0].cookies.is_none()`.
  - existing tests updated to `enqueue(url, None)`.
- [ ] **Step 2: Run, expect FAIL** — `cargo test -p clip2pod-rip --manifest-path src-tauri/Cargo.toml`.
- [ ] **Step 3: Implement**; update `rip_commands.rs` call to `enqueue(&url, None)`.
- [ ] **Step 4: Run, expect PASS** — same, plus `cargo check --workspace --manifest-path src-tauri/Cargo.toml`.
- [ ] **Step 5: Commit** — `feat(rip): jobs carry redacted cookies until they start`

---

### Task 3: App wiring — capture → enqueue → worker

**Files:**
- Modify: `src-tauri/src/capture.rs` (`CapturePayload`, `handle`, tests)
- Modify: `src-tauri/src/rip_commands.rs` (`do_enqueue`, `rip_enqueue`)
- Modify: `src-tauri/src/rip_worker.rs` (`run_job`)

**Interfaces:**
- Consumes: `Cookie`, `CookieJar`, `CookieFile`, `build_args(.., Option<&Path>)` (Task 1); `Queue::enqueue(url, Option<CookieJar>)`, `Job.cookies` (Task 2).
- Produces: `/capture` accepts optional `cookies: [{domain, hostOnly, path, secure, expirationDate?, name, value}]`; `pub fn do_enqueue(app: &AppHandle, url: String, cookies: Option<CookieJar>) -> CmdResult<String>`.

- [ ] **Step 1: Write failing tests** in `capture.rs` tests:
  - `payload_with_cookies_deserializes`: `{"url":"https://www.reddit.com/r/x/comments/a/b/","html":"","override_hint":"video","cookies":[{"domain":".reddit.com","hostOnly":false,"path":"/","secure":true,"expirationDate":1767225600.5,"name":"reddit_session","value":"v"},{"domain":"www.reddit.com","hostOnly":true,"path":"/","secure":false,"name":"s","value":"w"}]}` → `cookies` is `Some` with 2 entries, second `expiration_date == None`; `wants_rip` true.
  - `legacy_payload_has_no_cookies`: `{url, html}` → `cookies.is_none()`.
- [ ] **Step 2: Run, expect FAIL** — `cargo test -p clip2pod --lib capture --manifest-path src-tauri/Cargo.toml`.
- [ ] **Step 3: Implement**
  - `CapturePayload`: `#[serde(default)] cookies: Option<Vec<Cookie>>`. In `handle`, the rip branch calls `do_enqueue(app, url, payload.cookies.clone().filter(|c| !c.is_empty()).map(CookieJar))`; the narrate branch ignores cookies.
  - `rip_enqueue` (Tauri command) passes `None`.
  - `run_job`: before `build_args`, `let cookie_file = job.cookies.as_ref().map(|j| CookieFile::write(&j.0)).transpose().map_err(|e| format!("cannot write cookies file: {e}"))?;` — bind it for the whole function so `Drop` runs on every return path (RF2); pass `cookie_file.as_ref().map(|f| f.path())`.
- [ ] **Step 4: Run, expect PASS** — `cargo test --workspace --manifest-path src-tauri/Cargo.toml`.
- [ ] **Step 5: Commit** — `feat(capture): pass extension cookies through to rip jobs`

---

### Task 4: Extension — per-site permission, cookie collection, manifests

**Files:**
- Create: `extension/chrome/cookies.js`, `extension/firefox/cookies.js` (identical classic scripts)
- Create: `extension/test/cookies.test.mjs`
- Modify: `extension/{chrome,firefox}/background.js` (`send`, `contextMenus.onClicked`, `action.onClicked`; Chrome `importScripts("reddit.js", "cookies.js")`)
- Modify: `extension/{chrome,firefox}/manifest.json`
- Modify: `extension/test/reddit.test.mjs` byte-identity test → also `cookies.js`

**Interfaces:**
- Consumes: `/capture` `cookies` field (Task 3).
- Produces (top-level globals in `cookies.js`):
  - `siteBase(host: string) -> string | null` — registrable domain; `null` for IPs (v4 or bracketed v6), `localhost`, single-label hosts. Two-part suffix list: `co.uk, org.uk, ac.uk, gov.uk, com.au, net.au, org.au, co.nz, co.jp, co.in, com.br, co.za`.
  - `sitePattern(url: string) -> string | null` — `"*://*." + base + "/*"`.
  - `requestCookieAccess(url) -> Promise<boolean>` — must call `chrome.permissions.request` synchronously (no `await` before it); `null` pattern → `Promise.resolve(false)`; rejection → `false`.
  - `hasCookieAccess(url) -> Promise<boolean>` — `chrome.permissions.contains`, same null/rejection rules.
  - `async collectCookies(tab, access: Promise<boolean>) -> Array | undefined` — `undefined` unless `await access` is true; storeId = `tab.cookieStoreId ?? (await chrome.cookies.getAllCookieStores()).find(s => s.tabIds.includes(tab.id))?.id`; maps `getAll({url: tab.url, storeId})` to `{domain, hostOnly, path, secure, expirationDate, name, value}`; empty list or any thrown error → `undefined` (with `console.warn`).

- [ ] **Step 1: Write failing tests** (`cookies.test.mjs`, loading `../chrome/cookies.js` in a `vm` context with a stub `chrome` object):
  - `siteBase`: `www.reddit.com`/`old.reddit.com`/`reddit.com` → `reddit.com`; `m.youtube.com` → `youtube.com`; `www.bbc.co.uk` → `bbc.co.uk`; `127.0.0.1`, `localhost`, `[::1]`, `intranet` → `null` (RF5).
  - `sitePattern("https://old.reddit.com/r/x/comments/a/")` → `"*://*.reddit.com/*"`; `sitePattern("http://localhost:3000/")` → `null`.
  - `requestCookieAccess` calls `permissions.request` synchronously: stub records the call; assert the call happened before the returned promise is awaited, with `{origins:["*://*.reddit.com/*"]}`; stub rejecting → resolves `false`; localhost → `false` and no stub call (RF5).
  - `collectCookies`: access `false` → `undefined` and `getAll` not called (RF4); access true, Firefox-style tab `{cookieStoreId:"firefox-container-1"}` → `getAll` called with that `storeId`; Chrome-style tab `{id:7}` with stores `[{id:"0",tabIds:[1]},{id:"1",tabIds:[7]}]` → `storeId:"1"`; mapped objects have exactly the 7 keys; `getAll` throwing → `undefined` (RF4); empty result → `undefined`.
  - byte-identity test covers `cookies.js`.
- [ ] **Step 2: Run, expect FAIL** — `node --test extension/test/*.test.mjs`.
- [ ] **Step 3: Implement `cookies.js`**, copy to firefox/.
- [ ] **Step 4: Wire `background.js` (both builds, same shape)**
  - `send(tab, overrideHint, access)` — HTML path: `const cookies = access ? await collectCookies(tab, access) : undefined;` post `{ ...result, override_hint: overrideHint, ...(cookies && { cookies }) }`. Reddit-thread path untouched.
  - `contextMenus.onClicked`: video → `send(tab, "video", requestCookieAccess(tab.url))` with `requestCookieAccess` evaluated as the handler's first statement; article → `send(tab, "article")`.
  - `action.onClicked`: `send(tab, undefined, hasCookieAccess(tab.url))` (`tab.url` undefined → skip, pass nothing).
- [ ] **Step 5: Manifests** per Global Constraints (`cookies` permission, optional host entries, version `0.5.0`); Firefox `background.scripts`: `["reddit.js", "cookies.js", "background.js"]`.
- [ ] **Step 6: Verify** — `node --test extension/test/*.test.mjs` all pass; both manifests parse (`node -e "for (const b of ['chrome','firefox']) JSON.parse(require('fs').readFileSync('extension/'+b+'/manifest.json'))"`); `npx web-ext lint -s extension/firefox` → `errors 0`, no new warnings vs base.
- [ ] **Step 7: Commit** — `feat(extension): send the clicked site's cookies with rips`

---

### Task 5: Docs

**Files:**
- Modify: `README.md` (Privacy section; extension paragraph), `extension/README.md` (Use / Privacy, Layout)

- [ ] **Step 1:** Privacy wording: on Rip the extension sends that site's cookies (only those the browser would send to that page) to the app on `127.0.0.1`; used for that one download, then deleted; permission asked once per site on right-click Rip, revocable in the browser's extension settings; toolbar clicks reuse an existing grant and never prompt. Note that extension rips override any cookie flags in the saved Rip arguments for that run only.
- [ ] **Step 2:** `extension/README.md` Layout: `cookies.js` joins the byte-identical shared files; tested by `node --test extension/test/*.test.mjs`.
- [ ] **Step 3: Verify** every claim against Tasks 1–4 code; `git diff --check` clean.
- [ ] **Step 4: Commit** — `docs: rip with browser cookies (#21)` (PR body says `Closes #21`).
