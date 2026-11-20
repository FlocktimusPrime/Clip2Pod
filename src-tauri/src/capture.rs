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
// anything else. Pairing code: the origin check alone lets in every installed
// extension and every local process, so each capture must also carry the
// random code shown in Settings (`X-Clip2Pod-Token`), which the user pastes
// into the extension's options once.

use crate::state::AppState;
use clip2pod_rip::cookies::{Cookie, CookieJar};
use serde::Deserialize;
use std::io::Read;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

pub const CAPTURE_ADDR: &str = "127.0.0.1:4737";
pub const TOKEN_FILE: &str = "capture_token";
const TOKEN_HEADER: &str = "X-Clip2Pod-Token";
/// Rendered HTML of a long article is a few MB; anything far past that is abuse.
const MAX_BODY: usize = 32 * 1024 * 1024;
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

fn paired(given: Option<&str>, token: &str) -> bool {
    given.is_some_and(|g| crate::secret::same_token(g.trim(), token))
}

/// Read at most `MAX_BODY` bytes; `Err` carries the status and message to send.
fn read_capped(reader: impl Read, declared: Option<usize>) -> Result<String, (u16, &'static str)> {
    const TOO_BIG: (u16, &str) = (413, "page too large");
    if declared.is_some_and(|n| n > MAX_BODY) {
        return Err(TOO_BIG);
    }
    let mut body = String::new();
    reader
        .take(MAX_BODY as u64 + 1)
        .read_to_string(&mut body)
        .map_err(|_| (400, "unreadable body"))?;
    if body.len() > MAX_BODY {
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
        ("Access-Control-Allow-Headers", "Content-Type, X-Clip2Pod-Token"),
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
    if request.method() != &Method::Post || request.url() != "/capture" {
        return reply(request, 404, "not found");
    }
    // Checked before the body is read, so an unpaired caller can't hold a
    // worker with a slow upload.
    let token = app.state::<AppState>().capture_token.lock().unwrap().clone();
    if !paired(header_of(&request, TOKEN_HEADER).as_deref(), &token) {
        return reply(
            request,
            401,
            "not paired: paste the pairing code from Clip2Pod Settings into the extension's options",
        );
    }
    let declared = request.body_length();
    match read_capped(request.as_reader(), declared) {
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
    use super::{article_for, origin_allowed, paired, read_capped, wants_rip, CapturePayload, MAX_BODY};

    #[test]
    fn capture_needs_the_exact_pairing_code() {
        let t = "0123456789abcdef0123456789abcdef";
        assert!(paired(Some(t), t));
        assert!(paired(Some(&format!(" {t} ")), t));
        assert!(!paired(None, t));
        assert!(!paired(Some(""), t));
        assert!(!paired(Some("0123456789abcdef0123456789abcdee"), t));
    }

    #[test]
    fn body_is_capped() {
        assert_eq!(read_capped(&b"{}"[..], Some(2)).unwrap(), "{}");
        // Declared too big: refused without reading.
        assert_eq!(read_capped(&b""[..], Some(MAX_BODY + 1)).unwrap_err().0, 413);
        // Undeclared (chunked) and too big: refused after MAX_BODY + 1 bytes.
        let big = vec![b'a'; MAX_BODY + 10];
        assert_eq!(read_capped(&big[..], None).unwrap_err().0, 413);
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
