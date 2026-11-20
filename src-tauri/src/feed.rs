// LAN-facing podcast feed. One server, two feeds: `/tts/<token>/*` for narrated
// articles, `/video/<token>/*` for ripped audio. Read-only GET/HEAD. Binds all
// interfaces (unlike the loopback capture listener) so a phone on the same
// network can subscribe. There is deliberately no bare `/feed.xml` — the two
// feeds are separate subscriptions.
//
// Anyone on the same Wi-Fi can reach the port, so the URL itself is the
// secret: a random token, shared by both feeds and persisted in the config
// dir. Resetting it unsubscribes every device. Plain HTTP, so the token is only
// as private as the network: the server runs only while "Share feeds on this
// network" is on (off by default for new installs).

use crate::state::AppState;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

pub const FEED_PORT: u16 = 4738;
const FEED_ADDR: &str = "0.0.0.0:4738";
const COVER_TTS: &[u8] = include_bytes!("../assets/cover.png");
/// Bundled fallback; `/video/cover.jpg` prefers `rip_output_dir/cover.jpg` so
/// the user can swap artwork without a rebuild.
const COVER_VIDEO: &[u8] = include_bytes!("../assets/cover.jpg");

pub const TOKEN_FILE: &str = "feed_token";
/// Fixed handler pool: requests beyond this wait in tiny_http's queue instead
/// of each getting a thread, so a flood can't exhaust threads or memory.
const WORKERS: usize = 4;

/// Subscribe URL shown in the Feed dialog; `feed` is `tts` or `video`.
pub fn subscribe_url(feed: &str, token: &str) -> String {
    format!("http://{}:{FEED_PORT}/{feed}/{token}/feed.xml", lan_ip())
}

/// `/tts/<token>/rest` → `("tts", "rest")`. `None` for an unknown feed or a
/// wrong or missing token, so old token-less URLs and guesses get the same 404.
fn split_feed_path<'a>(url: &'a str, token: &str) -> Option<(&'a str, &'a str)> {
    let (feed, path) = url.strip_prefix('/')?.split_once('/')?;
    let (given, rest) = path.split_once('/')?;
    (matches!(feed, "tts" | "video") && crate::secret::same_token(given, token))
        .then_some((feed, rest))
}

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
/// checked) or the detected LAN address. `suffix` is `/tts/<token>` or
/// `/video/<token>`, so each core's `build_rss` produces correctly prefixed
/// enclosure/image URLs.
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

fn respond<R: std::io::Read>(request: Request, response: Response<R>, content_type: &str) {
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
    // Streamed from disk, not read into memory: episodes run to hundreds of MB.
    match std::fs::File::open(dir.join(name)) {
        Ok(file) => respond(request, Response::from_file(file), "audio/mpeg"),
        Err(_) => status(request, 404, "not found"),
    }
}

fn handle(app: &AppHandle, request: Request) {
    if request.method() != &Method::Get && request.method() != &Method::Head {
        return status(request, 405, "method not allowed");
    }
    let state = app.state::<AppState>();
    let token = state.feed_token.lock().unwrap().clone();
    let url = request.url().to_string();
    let Some((feed, rest)) = split_feed_path(&url, &token) else {
        return status(request, 404, "not found");
    };
    let base = base_url(&request, &format!("/{feed}/{token}"));
    match (feed, rest) {
        ("tts", "feed.xml") => {
            let xml = clip2pod_core::feed::build_rss(
                &clip2pod_core::feed::scan_episodes(&state.output_dir()),
                &base,
            );
            respond(
                request,
                Response::from_string(xml),
                "application/rss+xml; charset=utf-8",
            );
        }
        ("video", "feed.xml") => {
            let xml = clip2pod_rip::feed::build_rss(
                &clip2pod_rip::feed::scan_episodes(&state.rip_output_dir()),
                &base,
            );
            respond(
                request,
                Response::from_string(xml),
                "application/rss+xml; charset=utf-8",
            );
        }
        ("tts", "cover.png") => respond(
            request,
            Response::from_data(COVER_TTS.to_vec()),
            "image/png",
        ),
        ("video", "cover.jpg") => {
            let bytes = std::fs::read(state.rip_output_dir().join("cover.jpg"))
                .unwrap_or_else(|_| COVER_VIDEO.to_vec());
            respond(request, Response::from_data(bytes), "image/jpeg");
        }
        ("tts", path) if path.starts_with("audio/") => {
            let dir = state.output_dir();
            let Some(name) = clip2pod_core::feed::decode_audio_name(&path["audio/".len()..]) else {
                return status(request, 400, "bad filename");
            };
            let listed: Vec<String> = clip2pod_core::feed::scan_episodes(&dir)
                .into_iter()
                .map(|e| e.filename)
                .collect();
            serve_audio(request, &dir, &name, &listed);
        }
        ("video", path) if path.starts_with("audio/") => {
            let dir = state.rip_output_dir();
            let Some(name) = clip2pod_rip::feed::decode_audio_name(&path["audio/".len()..]) else {
                return status(request, 400, "bad filename");
            };
            let listed: Vec<String> = clip2pod_rip::feed::scan_episodes(&dir)
                .into_iter()
                .map(|e| e.filename)
                .collect();
            serve_audio(request, &dir, &name, &listed);
        }
        _ => status(request, 404, "not found"),
    }
}

/// The running server. tiny_http's own Drop wakes its accept thread by
/// connecting to the bound address, which fails for 0.0.0.0 on Windows and
/// leaves the port open until some client connects; so after dropping it,
/// connect to loopback ourselves.
pub struct FeedServer(Option<Server>);

impl FeedServer {
    fn get(&self) -> &Server {
        self.0.as_ref().expect("taken only in drop")
    }
}

impl Drop for FeedServer {
    fn drop(&mut self) {
        drop(self.0.take());
        let addr = (std::net::Ipv4Addr::LOCALHOST, FEED_PORT).into();
        let _ = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(500));
    }
}

/// Bind the LAN port and start the handler pool. No-op if already running.
pub fn start(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut running = state.feed_server.lock().unwrap();
    if running.is_some() {
        return Ok(());
    }
    let server = Arc::new(FeedServer(Some(Server::http(FEED_ADDR).map_err(|e| {
        format!("could not open port {FEED_PORT} ({e}) — if you just turned sharing off, try again in a moment")
    })?)));
    for _ in 0..WORKERS {
        let (server, app) = (server.clone(), app.clone());
        std::thread::spawn(move || {
            for request in server.get().incoming_requests() {
                handle(&app, request);
            }
        });
    }
    *running = Some(server);
    Ok(())
}

/// Stop accepting. Each worker exits after its current response; the port is
/// released when the last one drops its handle.
pub fn stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    let server = state.feed_server.lock().unwrap().take();
    if let Some(server) = server {
        // One unblock wakes one worker.
        for _ in 0..WORKERS {
            server.get().unblock();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn split_feed_path_requires_the_token() {
        assert_eq!(
            split_feed_path(&format!("/tts/{T}/feed.xml"), T),
            Some(("tts", "feed.xml"))
        );
        assert_eq!(
            split_feed_path(&format!("/video/{T}/audio/a.mp3"), T),
            Some(("video", "audio/a.mp3"))
        );
        assert_eq!(split_feed_path("/tts/feed.xml", T), None);
        assert_eq!(split_feed_path(&format!("/tts/{T}"), T), None);
        assert_eq!(split_feed_path(&format!("/tts/{T}x/feed.xml"), T), None);
        assert_eq!(split_feed_path(&format!("/other/{T}/feed.xml"), T), None);
        assert_eq!(
            split_feed_path("/tts/00000000000000000000000000000000/feed.xml", T),
            None
        );
    }
}
