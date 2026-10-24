//! TTS Client module

use super::{
    build_config_message, build_ssml_message, process_message,
    proxy::{ProxyAsyncStream, ProxyStream},
    websocket_connect, websocket_connect_async, websocket_connect_proxy,
    websocket_connect_proxy_async, AudioMetadata, ProcessedMessage, SpeechConfig, WebSocketStream,
    WebSocketStreamAsync,
};
use crate::error::Result;
use futures_util::{AsyncRead, AsyncWrite};
use std::io::{Read, Write};

/// Sync Client
pub struct MSEdgeTTSClient<T: Read + Write>(WebSocketStream<T>);

impl<T: Read + Write> MSEdgeTTSClient<T> {
    /// Synthesize text to speech with a [SpeechConfig] synchronously
    pub fn synthesize(&mut self, text: &str, config: &SpeechConfig) -> Result<SynthesizedAudio> {
        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.0.send(config_message)?;
        self.0.send(ssml_message)?;

        let mut audio_bytes = Vec::new();
        let mut audio_metadata = Vec::new();
        let mut turn_start = false;
        let mut response = false;
        let mut turn_end = false;
        loop {
            if turn_end {
                break;
            }

            let message = self.0.read()?;
            let message = process_message(message, &mut turn_start, &mut response, &mut turn_end)?;
            if let Some(message) = message {
                match message {
                    ProcessedMessage::AudioBytes(payload) => {
                        audio_bytes.push(payload);
                    }
                    ProcessedMessage::AudioMetadata(metadata) => {
                        audio_metadata.extend(metadata);
                    }
                }
            }
        }

        let audio_bytes = audio_bytes
            .iter()
            .flat_map(|(bytes, index)| &bytes[*index..])
            .copied()
            .collect();

        Ok(SynthesizedAudio {
            audio_format: config.audio_format.clone(),
            audio_bytes,
            audio_metadata,
        })
    }
}

impl MSEdgeTTSClient<std::net::TcpStream> {
    /// Apply a read timeout to the underlying socket.
    ///
    /// `synthesize` reads in a blocking loop until the service sends
    /// `turn.end`. When the link dies without the peer managing a FIN or RST
    /// — a dropped wifi connection, a sleeping laptop, a yanked cable — no
    /// error ever arrives and that read blocks until TCP keepalive gives up,
    /// which is two hours on Windows by default. A timeout converts that
    /// silence into `WouldBlock`/`TimedOut` so the caller can retry or give up.
    ///
    /// Clears any existing timeout when passed `None`.
    pub fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> std::io::Result<()> {
        match self.0.get_mut() {
            tungstenite::stream::MaybeTlsStream::Plain(tcp) => tcp.set_read_timeout(timeout),
            tungstenite::stream::MaybeTlsStream::NativeTls(tls) => {
                tls.get_mut().set_read_timeout(timeout)
            }
            // MaybeTlsStream is #[non_exhaustive]; an unreachable variant under
            // our feature set shouldn't be an error, just an un-timed socket.
            _ => Ok(()),
        }
    }
}

/// Async Client
pub struct MSEdgeTTSClientAsync<T>(WebSocketStreamAsync<T>);

impl<T: AsyncRead + AsyncWrite + Unpin> MSEdgeTTSClientAsync<T> {
    /// Synthesize text to speech with a [SpeechConfig] asynchronously
    pub async fn synthesize(
        &mut self,
        text: &str,
        config: &SpeechConfig,
    ) -> Result<SynthesizedAudio> {
        use futures_util::{SinkExt, StreamExt};

        let config_message = build_config_message(config);
        let ssml_message = build_ssml_message(text, config);
        self.0.send(config_message).await?;
        self.0.send(ssml_message).await?;

        let mut audio_bytes = Vec::new();
        let mut audio_metadata = Vec::new();
        let mut turn_start = false;
        let mut response = false;
        let mut turn_end = false;
        loop {
            if turn_end {
                break;
            }

            if let Some(message) = self.0.next().await {
                let message = message?;
                let response =
                    process_message(message, &mut turn_start, &mut response, &mut turn_end)?;
                if let Some(response) = response {
                    match response {
                        ProcessedMessage::AudioBytes(payload) => {
                            audio_bytes.push(payload);
                        }
                        ProcessedMessage::AudioMetadata(metadata) => {
                            audio_metadata.extend(metadata);
                        }
                    }
                }
            }
        }

        let audio_bytes = audio_bytes
            .iter()
            .flat_map(|(bytes, index)| &bytes[*index..])
            .copied()
            .collect();

        Ok(SynthesizedAudio {
            audio_format: config.audio_format.clone(),
            audio_bytes,
            audio_metadata,
        })
    }
}

/// Synthesized Audio and Metadata
#[derive(Debug)]
pub struct SynthesizedAudio {
    pub audio_format: String,
    pub audio_bytes: Vec<u8>,
    pub audio_metadata: Vec<AudioMetadata>,
}

/// Create Sync TTS [Client](MSEdgeTTSClient)
pub fn connect() -> Result<MSEdgeTTSClient<std::net::TcpStream>> {
    Ok(MSEdgeTTSClient(websocket_connect()?))
}

/// Create Sync TTS [Client](MSEdgeTTSClient) with proxy
///
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy.  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub fn connect_proxy(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<MSEdgeTTSClient<ProxyStream>> {
    Ok(MSEdgeTTSClient(websocket_connect_proxy(
        proxy, username, password,
    )?))
}

/// Create Async TTS [Client](MSEdgeTTSClientAsync)
pub async fn connect_async() -> Result<MSEdgeTTSClientAsync<async_std::net::TcpStream>> {
    Ok(MSEdgeTTSClientAsync(websocket_connect_async().await?))
}

/// Create Async TTS [Client](MSEdgeTTSClientAsync) with proxy
///
/// The proxy protocol is specified by the URI scheme.
///
/// `http`: Proxy. Default when no scheme is specified.  
/// `https`: HTTPS Proxy.  
/// `socks4`: SOCKS4 Proxy.  
/// `socks4a`: SOCKS4a Proxy. Proxy resolves URL hostname.  
/// `socks5`: SOCKS5 Proxy.  
/// `socks5h`: SOCKS5 Proxy. Proxy resolves URL hostname.  
pub async fn connect_proxy_async(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<MSEdgeTTSClientAsync<ProxyAsyncStream>> {
    Ok(MSEdgeTTSClientAsync(
        websocket_connect_proxy_async(proxy, username, password).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A socket that is open but silent is the shape a dropped wifi link
    /// leaves behind: no FIN, no RST, nothing to read, and `read` blocking on
    /// TCP keepalive. Assert the timeout reaches the real socket and turns
    /// that silence into an error.
    #[test]
    fn read_timeout_applies_to_a_live_but_silent_socket() {
        use std::net::{TcpListener, TcpStream};
        use std::time::{Duration, Instant};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        // Accept and then hold the connection open without writing. Dropping
        // it here would send a FIN and `read` would return cleanly, testing
        // nothing.
        let server = std::thread::spawn(move || {
            let held = listener.accept().unwrap();
            std::thread::sleep(Duration::from_secs(2));
            drop(held);
        });

        let tcp = TcpStream::connect(addr).unwrap();
        let mut client = MSEdgeTTSClient(tungstenite::WebSocket::from_raw_socket(
            tungstenite::stream::MaybeTlsStream::Plain(tcp),
            tungstenite::protocol::Role::Client,
            None,
        ));
        client
            .set_read_timeout(Some(Duration::from_millis(150)))
            .unwrap();

        let started = Instant::now();
        let err = client.0.read().unwrap_err();
        let waited = started.elapsed();

        match err {
            tungstenite::Error::Io(e) => assert!(
                // Unix reports WouldBlock, Windows TimedOut.
                matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ),
                "expected a timeout, got {e:?}"
            ),
            other => panic!("expected an io timeout, got {other:?}"),
        }
        assert!(
            waited < Duration::from_secs(1),
            "read should give up on the timeout, not keep waiting ({waited:?})"
        );

        server.join().unwrap();
    }
}
