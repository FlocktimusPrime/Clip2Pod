// Loopback listener for the browser extension. One endpoint for both modes: the
// extension sends the page URL plus either its rendered HTML (the user's
// logged-in session, so paywalled article text is included) or, for pages it
// can extract itself (Reddit threads), a finished `prepared` title/author/text
// that skips Readability. It may also send an override telling us which mode
// the user picked from the context menu.
//
// Routing: an explicit override wins; a prepared payload is an article;
// otherwise a known video host goes to RIP and everything else is treated as an
// article extracted from the HTML.
//
// Two gates. Origin: browsers attach `Origin` to cross-origin fetches, so a web
// page trying to POST here arrives with its own origin and is refused before
// anything else. Pairing: the origin check alone lets in every installed
// extension and every local process, so captures must prove knowledge of the
// random code shown in Settings, which the user pastes into the extension once.
//
// The code itself never travels, both ways, because when Clip2Pod isn't running
// any local program can listen on this port and would receive whatever the
// extension sends (page HTML, site cookies, the code):
//   1. POST /hello {"nonce": n}  →  {"proof": HMAC(code, SERVER+n), "challenge": c}
//      The extension checks the proof and sends nothing if it's wrong.
//   2. POST /capture with `X-Clip2Pod-Auth: c.HMAC(code, CLIENT+c)`.
//      `c` is single-use and expires, so a captured header can't be replayed.

use crate::secret::{new_token, same_token, to_hex};
use crate::state::AppState;
use clip2pod_rip::cookies::{Cookie, CookieJar};
use ring::hmac;
use serde::Deserialize;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

pub const CAPTURE_ADDR: &str = "127.0.0.1:4737";
pub const TOKEN_FILE: &str = "capture_token";
const AUTH_HEADER: &str = "X-Clip2Pod-Auth";
/// Domain separation: a server proof can never be replayed as a client one.
const SERVER_LABEL: &str = "clip2pod-server:";
const CLIENT_LABEL: &str = "clip2pod-client:";
const CHALLENGE_TTL: Duration = Duration::from_secs(60);
/// Outstanding challenges. Anyone past the origin gate can call /hello, so the
/// list is capped (oldest dropped): flooding it can only make a real capture
/// retry, never grow memory.
const MAX_PENDING: usize = 16;
static PENDING: Mutex<VecDeque<(String, Instant)>> = Mutex::new(VecDeque::new());
/// Rendered HTML of a long article is a few MB; anything far past that is abuse.
const MAX_BODY: usize = 32 * 1024 * 1024;
const MAX_HELLO: usize = 1024;
/// Request handlers. A slow upload ties one up, so more than one keeps the
/// listener responsive.
const WORKERS: usize = 2;

#[derive(Deserialize)]
struct Prepared {
    title: String,
    author: String,
    text: String,
}

#[derive(Deserialize)]
struct CapturePayload {
    url: String,
    /// Absent when the extension sends `prepared` text instead.
    #[serde(default)]
    html: String,
    /// Finished article text from an extension-side extractor; skips Readability.
    #[serde(default)]
    prepared: Option<Prepared>,
    /// "article" or "video" from the extension's context menu; absent for a
    /// plain toolbar click (auto-route).
    #[serde(default)]
    override_hint: Option<String>,
    /// Site cookies for a rip; ignored by the narrate path.
    #[serde(default)]
    cookies: Option<Vec<Cookie>>,
}

#[derive(serde::Serialize, Clone)]
struct CapturedArticle {
    title: String,
    author: String,
    text: String,
    url: String,
    /// True when the text was prepared by the extension rather than extracted here.
    reset_gender: bool,
}

fn origin_allowed(origin: Option<&str>) -> bool {
    match origin {
        None => true,
        Some(o) => o.starts_with("chrome-extension://") || o.starts_with("moz-extension://"),
    }
}

fn header_of(request: &Request, name: &'static str) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|h| h.field.equiv(name))
        .map(|h| h.value.to_string())
}

/// Lowercase hex HMAC-SHA256 of `label + data`, keyed by the pairing code.
/// extension/chrome/pairing.js computes the same thing.
fn mac(code: &str, label: &str, data: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, code.as_bytes());
    to_hex(hmac::sign(&key, format!("{label}{data}").as_bytes()).as_ref())
}

fn prune(pending: &mut VecDeque<(String, Instant)>, now: Instant) {
    pending.retain(|(_, t)| now.duration_since(*t) < CHALLENGE_TTL);
}

fn issue(pending: &mut VecDeque<(String, Instant)>, challenge: String, now: Instant) {
    prune(pending, now);
    if pending.len() >= MAX_PENDING {
        pending.pop_front();
    }
    pending.push_back((challenge, now));
}

/// Step 1: prove we know the code, and hand out a fresh challenge.
fn hello(code: &str, body: &str, pending: &mut VecDeque<(String, Instant)>, now: Instant) -> (u16, String) {
    #[derive(Deserialize)]
    struct Hello {
        nonce: String,
    }
    let nonce = match serde_json::from_str::<Hello>(body) {
        Ok(h) if crate::secret::is_valid_token(&h.nonce) => h.nonce,
        _ => return (400, "bad hello".into()),
    };
    let challenge = new_token();
    issue(pending, challenge.clone(), now);
    let reply = serde_json::json!({
        "proof": mac(code, SERVER_LABEL, &nonce),
        "challenge": challenge,
    });
    (200, reply.to_string())
}

/// Step 2: `header` is `challenge.signature`. The signature is checked before
/// the challenge is consumed, so a forged header can't burn a real one.
fn authorized(
    header: Option<&str>,
    code: &str,
    pending: &mut VecDeque<(String, Instant)>,
    now: Instant,
) -> bool {
    let Some((challenge, sig)) = header.and_then(|h| h.trim().split_once('.')) else {
        return false;
    };
    if !same_token(&mac(code, CLIENT_LABEL, challenge), sig) {
        return false;
    }
    prune(pending, now);
    match pending.iter().position(|(c, _)| same_token(c, challenge)) {
        Some(i) => {
            pending.remove(i);
            true
        }
        None => false,
    }
}

/// Read at most `limit` bytes; `Err` carries the status and message to send.
fn read_capped(
    reader: impl Read,
    declared: Option<usize>,
    limit: usize,
) -> Result<String, (u16, &'static str)> {
    const TOO_BIG: (u16, &str) = (413, "page too large");
    if declared.is_some_and(|n| n > limit) {
        return Err(TOO_BIG);
    }
    let mut body = String::new();
    reader
        .take(limit as u64 + 1)
        .read_to_string(&mut body)
        .map_err(|_| (400, "unreadable body"))?;
    if body.len() > limit {
        return Err(TOO_BIG);
    }
    Ok(body)
}

fn with_cors(
    mut response: Response<std::io::Cursor<Vec<u8>>>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    for (name, value) in [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type, X-Clip2Pod-Auth"),
    ] {
        if let Ok(h) = Header::from_bytes(name.as_bytes(), value.as_bytes()) {
            response.add_header(h);
        }
    }
    response
}

fn handle(app: &AppHandle, body: &str) -> (u16, String) {
    let payload: CapturePayload = match serde_json::from_str(body) {
        Ok(p) => p,
        Err(e) => return (400, format!("bad request: {e}")),
    };

    let result = if wants_rip(&payload) {
        crate::rip_commands::do_enqueue(
            app,
            payload.url.clone(),
            payload
                .cookies
                .clone()
                .filter(|c| !c.is_empty())
                .map(CookieJar),
        )
        .map(|_| ("rip", "queued"))
    } else {
        route_article(app, &payload).map(|_| ("narrate", "captured"))
    };

    match result {
        Ok((tab, msg)) => {
            let _ = app.emit("switch-tab", tab);
            crate::tray::show_main(app);
            (200, msg.to_string())
        }
        Err(e) => (422, e),
    }
}

fn wants_rip(p: &CapturePayload) -> bool {
    match p.override_hint.as_deref() {
        Some("video") => true,
        Some("article") => false,
        _ if p.prepared.is_some() => false,
        _ => clip2pod_rip::ytdlp::rippable_host(&p.url),
    }
}

fn article_for(p: &CapturePayload) -> Result<CapturedArticle, String> {
    if let Some(prep) = &p.prepared {
        return Ok(CapturedArticle {
            title: prep.title.clone(),
            author: prep.author.clone(),
            text: prep.text.clone(),
            url: p.url.clone(),
            reset_gender: true,
        });
    }
    let extracted =
        clip2pod_core::extract::extract_from_html(&p.html, &p.url).map_err(|e| e.to_string())?;
    Ok(CapturedArticle {
        title: extracted.title,
        author: extracted.author,
        text: extracted.text,
        url: p.url.clone(),
        reset_gender: false,
    })
}

fn route_article(app: &AppHandle, payload: &CapturePayload) -> Result<(), String> {
    let article = article_for(payload)?;
    let _ = app.emit("article-captured", &article);
    Ok(())
}

fn reply(request: Request, status: u16, message: &str) {
    let _ = request.respond(with_cors(
        Response::from_string(message).with_status_code(status),
    ));
}

fn handle_request(app: &AppHandle, mut request: Request) {
    if !origin_allowed(header_of(&request, "Origin").as_deref()) {
        let _ = request.respond(Response::from_string("forbidden").with_status_code(403));
        return;
    }
    if request.method() == &Method::Options {
        let _ = request.respond(with_cors(Response::from_data(Vec::new())));
        return;
    }
    let is_hello = match (request.method(), request.url()) {
        (Method::Post, "/hello") => true,
        (Method::Post, "/capture") => false,
        _ => return reply(request, 404, "not found"),
    };
    let code = app.state::<AppState>().capture_token.lock().unwrap().clone();
    let declared = request.body_length();
    if is_hello {
        let (status, message) = match read_capped(request.as_reader(), declared, MAX_HELLO) {
            Ok(body) => hello(&code, &body, &mut PENDING.lock().unwrap(), Instant::now()),
            Err((status, message)) => (status, message.to_string()),
        };
        return reply(request, status, &message);
    }
    // Checked before the body is read, so an unpaired caller can't hold a
    // worker with a slow upload.
    let auth = header_of(&request, AUTH_HEADER);
    if !authorized(auth.as_deref(), &code, &mut PENDING.lock().unwrap(), Instant::now()) {
        return reply(
            request,
            401,
            "not paired: paste the pairing code from Clip2Pod Settings into the extension's options",
        );
    }
    match read_capped(request.as_reader(), declared, MAX_BODY) {
        Ok(body) => {
            let (status, message) = handle(app, &body);
            reply(request, status, &message);
        }
        Err((status, message)) => reply(request, status, message),
    }
}

pub fn serve(app: AppHandle) {
    let server = match Server::http(CAPTURE_ADDR) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("capture listener failed to bind {CAPTURE_ADDR}: {e}");
            return;
        }
    };
    for _ in 0..WORKERS {
        let (server, app) = (server.clone(), app.clone());
        std::thread::spawn(move || {
            for request in server.incoming_requests() {
                handle_request(&app, request);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::time::{Duration, Instant};

    const CODE: &str = "0123456789abcdef0123456789abcdef";
    const NONCE: &str = "fedcba9876543210fedcba9876543210";

    #[test]
    fn mac_is_standard_hmac_sha256_and_matches_the_extension() {
        // RFC 4231 test case 2.
        assert_eq!(
            mac("Jefe", "", "what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        // Same vector as extension/test/pairing.test.mjs.
        assert_eq!(
            mac(CODE, CLIENT_LABEL, NONCE),
            "0779ba6aa5d75bc7771501b5a2327ec77726a163b08002249db8a42f9bcc7749"
        );
    }

    /// Run /hello, check its proof, return the challenge.
    fn handshake(pending: &mut VecDeque<(String, Instant)>, now: Instant) -> String {
        let (status, body) = hello(CODE, &format!(r#"{{"nonce":"{NONCE}"}}"#), pending, now);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["proof"], mac(CODE, SERVER_LABEL, NONCE));
        v["challenge"].as_str().unwrap().to_string()
    }

    fn header(code: &str, challenge: &str) -> String {
        format!("{challenge}.{}", mac(code, CLIENT_LABEL, challenge))
    }

    #[test]
    fn signed_challenge_is_accepted_once() {
        let (mut p, now) = (VecDeque::new(), Instant::now());
        let c = handshake(&mut p, now);
        let h = header(CODE, &c);
        assert!(authorized(Some(&h), CODE, &mut p, now));
        assert!(!authorized(Some(&h), CODE, &mut p, now), "replay");
    }

    #[test]
    fn wrong_code_is_refused_without_burning_the_challenge() {
        let (mut p, now) = (VecDeque::new(), Instant::now());
        let c = handshake(&mut p, now);
        let other = "ffffffffffffffffffffffffffffffff";
        assert!(!authorized(Some(&header(other, &c)), CODE, &mut p, now));
        assert!(authorized(Some(&header(CODE, &c)), CODE, &mut p, now));
    }

    #[test]
    fn missing_unknown_or_expired_challenges_are_refused() {
        let (mut p, now) = (VecDeque::new(), Instant::now());
        assert!(!authorized(None, CODE, &mut p, now));
        assert!(!authorized(Some("garbage"), CODE, &mut p, now));
        // Correctly signed, but never issued.
        assert!(!authorized(Some(&header(CODE, NONCE)), CODE, &mut p, now));
        let c = handshake(&mut p, now);
        let later = now + CHALLENGE_TTL + Duration::from_secs(1);
        assert!(!authorized(Some(&header(CODE, &c)), CODE, &mut p, later));
    }

    #[test]
    fn hello_rejects_bad_nonces_and_caps_pending() {
        let (mut p, now) = (VecDeque::new(), Instant::now());
        for body in ["", "{}", r#"{"nonce":"short"}"#, r#"{"nonce":"FEDCBA9876543210FEDCBA9876543210"}"#] {
            assert_eq!(hello(CODE, body, &mut p, now).0, 400, "{body}");
        }
        let first = handshake(&mut p, now);
        for _ in 0..MAX_PENDING {
            handshake(&mut p, now);
        }
        assert_eq!(p.len(), MAX_PENDING);
        assert!(!authorized(Some(&header(CODE, &first)), CODE, &mut p, now), "evicted");
    }

    #[test]
    fn body_is_capped() {
        assert_eq!(read_capped(&b"{}"[..], Some(2), MAX_BODY).unwrap(), "{}");
        // Declared too big: refused without reading.
        assert_eq!(read_capped(&b""[..], Some(MAX_BODY + 1), MAX_BODY).unwrap_err().0, 413);
        // Undeclared (chunked) and too big: refused after limit + 1 bytes.
        let big = vec![b'a'; MAX_HELLO + 10];
        assert_eq!(read_capped(&big[..], None, MAX_HELLO).unwrap_err().0, 413);
    }

    #[test]
    fn prepared_payload_needs_no_html() {
        let p: CapturePayload = serde_json::from_str(
            r#"{"url":"https://www.reddit.com/comments/abc","prepared":{"title":"T from r/x","author":"u/op","text":"body"}}"#,
        )
        .unwrap();
        assert_eq!(p.html, "");
        let a = article_for(&p).unwrap();
        assert_eq!(
            (a.title.as_str(), a.author.as_str(), a.text.as_str()),
            ("T from r/x", "u/op", "body")
        );
        assert!(a.reset_gender);
    }

    #[test]
    fn legacy_payload_still_parses() {
        let p: CapturePayload =
            serde_json::from_str(r#"{"url":"https://e.com/a","html":"<html></html>"}"#).unwrap();
        assert!(p.prepared.is_none());
        assert!(!wants_rip(&p));
    }

    #[test]
    fn payload_with_cookies_deserializes() {
        let p: CapturePayload = serde_json::from_str(
            r#"{"url":"https://www.reddit.com/r/x/comments/a/b/","html":"","override_hint":"video","cookies":[{"domain":".reddit.com","hostOnly":false,"path":"/","secure":true,"expirationDate":1767225600.5,"name":"reddit_session","value":"v"},{"domain":"www.reddit.com","hostOnly":true,"path":"/","secure":false,"name":"s","value":"w"}]}"#,
        )
        .unwrap();
        let c = p.cookies.as_ref().unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[1].expiration_date, None);
        assert!(wants_rip(&p));
    }

    #[test]
    fn legacy_payload_has_no_cookies() {
        let p: CapturePayload =
            serde_json::from_str(r#"{"url":"https://e.com/a","html":"<html></html>"}"#).unwrap();
        assert!(p.cookies.is_none());
    }

    #[test]
    fn prepared_beats_rippable_host() {
        let p: CapturePayload = serde_json::from_str(
            r#"{"url":"https://www.youtube.com/watch?v=x","prepared":{"title":"t","author":"a","text":"b"}}"#,
        )
        .unwrap();
        assert!(!wants_rip(&p));
        let forced: CapturePayload = serde_json::from_str(
            r#"{"url":"https://e.com/a","override_hint":"video","prepared":{"title":"t","author":"a","text":"b"}}"#,
        )
        .unwrap();
        assert!(wants_rip(&forced));
    }

    #[test]
    fn html_path_keeps_reader_gender() {
        let para = "The committee met on Tuesday to discuss the new budget proposal. \
            Members argued for several hours about spending priorities and timelines. \
            In the end they agreed to revisit the question next month after more data arrives. \
            Observers noted that the tone of the debate was calmer than in previous sessions. ";
        let html = format!(
            "<html><head><title>Budget talks</title></head><body><article><h1>Budget talks</h1><p>{para}</p><p>{para}</p></article></body></html>"
        );
        let p = CapturePayload {
            url: "https://e.com/a".into(),
            html,
            override_hint: None,
            prepared: None,
            cookies: None,
        };
        let a = article_for(&p).unwrap();
        assert!(!a.reset_gender);
    }

    #[test]
    fn only_extensions_and_originless_callers_get_in() {
        assert!(origin_allowed(None));
        assert!(origin_allowed(Some("chrome-extension://abcdefghijklmnop")));
        assert!(origin_allowed(Some("moz-extension://1234-5678")));
        assert!(!origin_allowed(Some("https://evil.example")));
        assert!(!origin_allowed(Some("http://127.0.0.1:4737")));
        assert!(!origin_allowed(Some("null")));
        assert!(!origin_allowed(Some("")));
    }
}
