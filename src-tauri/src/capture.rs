// Loopback listener for the browser extension: receives the rendered page
// HTML (user's logged-in session, so paywalled content included) and turns
// it into editor content via the shared readability pipeline.

use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager};
use tiny_http::{Header, Method, Response, Server};

pub const CAPTURE_ADDR: &str = "127.0.0.1:4737";

#[derive(Deserialize)]
struct CapturePayload {
    url: String,
    html: String,
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
    match clip2pod_core::extract::extract_from_html(&payload.html, &payload.url) {
        Ok(extracted) => {
            let _ = app.emit(
                "article-captured",
                &CapturedArticle {
                    title: extracted.title,
                    author: extracted.author,
                    text: extracted.text,
                    url: payload.url.clone(),
                },
            );
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            (200, "captured".into())
        }
        Err(e) => (422, e.to_string()),
    }
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
                let _ = request.respond(with_cors(Response::from_string("not found").with_status_code(404)));
                continue;
            }
            let mut body = String::new();
            if request.as_reader().read_to_string(&mut body).is_err() {
                let _ = request.respond(with_cors(Response::from_string("unreadable body").with_status_code(400)));
                continue;
            }
            let (status, message) = handle(&app, &body);
            let _ = request.respond(with_cors(Response::from_string(message).with_status_code(status)));
        }
    });
}
