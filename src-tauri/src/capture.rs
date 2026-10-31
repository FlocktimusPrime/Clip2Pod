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
// Origin gate: browsers attach `Origin` to cross-origin fetches, so a web page
// trying to POST here arrives with its own origin and is refused. Extensions
// arrive as `chrome-extension://` / `moz-extension://`; curl and local scripts
// send no Origin at all and are allowed (anything that can run them already
// runs as the user).

use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use tiny_http::{Header, Method, Response, Server};

pub const CAPTURE_ADDR: &str = "127.0.0.1:4737";

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

fn origin_of(request: &tiny_http::Request) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Origin"))
        .map(|h| h.value.to_string())
}

fn with_cors(mut response: Response<std::io::Cursor<Vec<u8>>>) -> Response<std::io::Cursor<Vec<u8>>> {
    for (name, value) in [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "POST, OPTIONS"),
        ("Access-Control-Allow-Headers", "Content-Type"),
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
        crate::rip_commands::do_enqueue(app, payload.url.clone()).map(|_| ("rip", "queued"))
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

pub fn serve(app: AppHandle) {
    std::thread::spawn(move || {
        let server = match Server::http(CAPTURE_ADDR) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("capture listener failed to bind {CAPTURE_ADDR}: {e}");
                return;
            }
        };
        for mut request in server.incoming_requests() {
            if !origin_allowed(origin_of(&request).as_deref()) {
                let _ = request.respond(Response::from_string("forbidden").with_status_code(403));
                continue;
            }
            if request.method() == &Method::Options {
                let _ = request.respond(with_cors(Response::from_data(Vec::new())));
                continue;
            }
            if request.method() != &Method::Post || request.url() != "/capture" {
                let _ = request
                    .respond(with_cors(Response::from_string("not found").with_status_code(404)));
                continue;
            }
            let mut body = String::new();
            if request.as_reader().read_to_string(&mut body).is_err() {
                let _ = request.respond(
                    with_cors(Response::from_string("unreadable body").with_status_code(400)),
                );
                continue;
            }
            let (status, message) = handle(&app, &body);
            let _ =
                request.respond(with_cors(Response::from_string(message).with_status_code(status)));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{article_for, origin_allowed, wants_rip, CapturePayload};

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
