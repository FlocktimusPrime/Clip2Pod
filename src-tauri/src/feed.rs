// LAN-facing podcast feed: read-only GET routes for the RSS document, audio
// files and the bundled cover. Unlike the loopback capture listener this
// binds all interfaces so a phone on the same network can subscribe.

use crate::state::AppState;
use clip2pod_core::feed::{build_rss, decode_audio_name, scan_episodes};
use std::io::Cursor;
use tauri::{AppHandle, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

pub const FEED_PORT: u16 = 4738;
const FEED_ADDR: &str = "0.0.0.0:4738";
const COVER: &[u8] = include_bytes!("../assets/cover.png");

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

fn output_dir(app: &AppHandle) -> std::path::PathBuf {
    app.state::<AppState>().output_dir()
}

fn handle(app: &AppHandle, request: Request) {
    if request.method() != &Method::Get && request.method() != &Method::Head {
        return status(request, 405, "method not allowed");
    }
    let url = request.url().to_string();
    match url.as_str() {
        "/feed.xml" => {
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
            let xml = build_rss(&scan_episodes(&output_dir(app)), &format!("http://{host}"));
            respond(
                request,
                Response::from_string(xml),
                "application/rss+xml; charset=utf-8",
            );
        }
        "/cover.png" => respond(request, Response::from_data(COVER.to_vec()), "image/png"),
        path if path.starts_with("/audio/") => {
            let Some(name) = decode_audio_name(&path["/audio/".len()..]) else {
                return status(request, 400, "bad filename");
            };
            let dir = output_dir(app);
            // only files the feed actually lists are downloadable — the
            // fallback output dir can be $HOME, never expose it wholesale
            if !scan_episodes(&dir).iter().any(|e| e.filename == name) {
                return status(request, 404, "not found");
            }
            match std::fs::read(dir.join(&name)) {
                Ok(bytes) => respond(request, Response::from_data(bytes), "audio/mpeg"),
                Err(_) => status(request, 404, "not found"),
            }
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
