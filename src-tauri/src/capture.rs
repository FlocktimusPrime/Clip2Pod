// Loopback listener for the browser extension. One endpoint for both modes: the
// extension always sends the page URL plus its rendered HTML (the user's
// logged-in session, so paywalled article text is included) and, optionally, an
// override telling us which mode the user picked from the context menu.
//
// Routing: an explicit override wins; otherwise a known video host goes to RIP
// and everything else is treated as an article. The extension always includes
// the HTML, so the article path always has what it needs.

use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use tiny_http::{Header, Method, Response, Server};

pub const CAPTURE_ADDR: &str = "127.0.0.1:4737";

#[derive(Deserialize)]
struct CapturePayload {
    url: String,
    html: String,
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

    let to_rip = match payload.override_hint.as_deref() {
        Some("video") => true,
        Some("article") => false,
        _ => ytdlfeed_core::ytdlp::rippable_host(&payload.url),
    };

    let result = if to_rip {
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

fn route_article(app: &AppHandle, payload: &CapturePayload) -> Result<(), String> {
    let extracted = clip2pod_core::extract::extract_from_html(&payload.html, &payload.url)
        .map_err(|e| e.to_string())?;
    let _ = app.emit(
        "article-captured",
        &CapturedArticle {
            title: extracted.title,
            author: extracted.author,
            text: extracted.text,
            url: payload.url.clone(),
        },
    );
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
