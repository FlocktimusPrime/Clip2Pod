// LAN-facing podcast feed. One server, two feeds: `/tts/*` for narrated
// articles, `/video/*` for ripped audio. Read-only GET/HEAD. Binds all
// interfaces (unlike the loopback capture listener) so a phone on the same
// network can subscribe. There is deliberately no bare `/feed.xml` — the two
// feeds are separate subscriptions.

use crate::state::AppState;
use std::io::Cursor;
use std::path::Path;
use tauri::{AppHandle, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

pub const FEED_PORT: u16 = 4738;
const FEED_ADDR: &str = "0.0.0.0:4738";
const COVER_TTS: &[u8] = include_bytes!("../assets/cover.png");
/// Bundled fallback; `/video/cover.jpg` prefers `rip_output_dir/cover.jpg` so
/// the user can swap artwork without a rebuild.
const COVER_VIDEO: &[u8] = include_bytes!("../assets/cover.jpg");

/// Best-effort LAN address: a connected UDP socket picks the outbound
/// interface without sending a packet. Loopback fallback keeps the URL
/// usable for desktop testing.
pub fn lan_ip() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".into())
}

/// `http://<host><suffix>` where host is the request's `Host` header (charset
/// checked) or the detected LAN address. `suffix` is `/tts` or `/video`, so
/// each core's `build_rss` produces correctly prefixed enclosure/image URLs.
fn base_url(request: &Request, suffix: &str) -> String {
    let host = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Host"))
        .map(|h| h.value.to_string())
        .filter(|h| {
            h.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-:[]".contains(&b))
        })
        .unwrap_or_else(|| format!("{}:{FEED_PORT}", lan_ip()));
    format!("http://{host}{suffix}")
}

fn respond(request: Request, response: Response<Cursor<Vec<u8>>>, content_type: &str) {
    let with_type = match Header::from_bytes(b"Content-Type", content_type.as_bytes()) {
        Ok(h) => response.with_header(h),
        Err(_) => response,
    };
    let _ = request.respond(with_type);
}

fn status(request: Request, code: u16, message: &str) {
    let _ = request.respond(Response::from_string(message).with_status_code(code));
}

/// Serve one mp3 by name, but only if the feed for that dir actually lists it —
/// the fallback output dir can be `$HOME`, never expose it wholesale.
fn serve_audio(request: Request, dir: &Path, name: &str, listed: &[String]) {
    if !listed.iter().any(|f| f == name) {
        return status(request, 404, "not found");
    }
    match std::fs::read(dir.join(name)) {
        Ok(bytes) => respond(request, Response::from_data(bytes), "audio/mpeg"),
        Err(_) => status(request, 404, "not found"),
    }
}

fn handle(app: &AppHandle, request: Request) {
    if request.method() != &Method::Get && request.method() != &Method::Head {
        return status(request, 405, "method not allowed");
    }
    let state = app.state::<AppState>();
    let url = request.url().to_string();
    match url.as_str() {
        "/tts/feed.xml" => {
            let xml = clip2pod_core::feed::build_rss(
                &clip2pod_core::feed::scan_episodes(&state.output_dir()),
                &base_url(&request, "/tts"),
            );
            respond(
                request,
                Response::from_string(xml),
                "application/rss+xml; charset=utf-8",
            );
        }
        "/video/feed.xml" => {
            let xml = ytdlfeed_core::feed::build_rss(
                &ytdlfeed_core::feed::scan_episodes(&state.rip_output_dir()),
                &base_url(&request, "/video"),
            );
            respond(
                request,
                Response::from_string(xml),
                "application/rss+xml; charset=utf-8",
            );
        }
        "/tts/cover.png" => {
            respond(request, Response::from_data(COVER_TTS.to_vec()), "image/png")
        }
        "/video/cover.jpg" => {
            let bytes = std::fs::read(state.rip_output_dir().join("cover.jpg"))
                .unwrap_or_else(|_| COVER_VIDEO.to_vec());
            respond(request, Response::from_data(bytes), "image/jpeg");
        }
        path if path.starts_with("/tts/audio/") => {
            let dir = state.output_dir();
            let Some(name) = clip2pod_core::feed::decode_audio_name(&path["/tts/audio/".len()..])
            else {
                return status(request, 400, "bad filename");
            };
            let listed: Vec<String> = clip2pod_core::feed::scan_episodes(&dir)
                .into_iter()
                .map(|e| e.filename)
                .collect();
            serve_audio(request, &dir, &name, &listed);
        }
        path if path.starts_with("/video/audio/") => {
            let dir = state.rip_output_dir();
            let Some(name) =
                ytdlfeed_core::feed::decode_audio_name(&path["/video/audio/".len()..])
            else {
                return status(request, 400, "bad filename");
            };
            let listed: Vec<String> = ytdlfeed_core::feed::scan_episodes(&dir)
                .into_iter()
                .map(|e| e.filename)
                .collect();
            serve_audio(request, &dir, &name, &listed);
        }
        _ => status(request, 404, "not found"),
    }
}

pub fn serve(app: AppHandle) {
    std::thread::spawn(move || {
        let server = match Server::http(FEED_ADDR) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("feed server failed to bind {FEED_ADDR}: {e}");
                return;
            }
        };
        for request in server.incoming_requests() {
            let app = app.clone();
            std::thread::spawn(move || handle(&app, request));
        }
    });
}
