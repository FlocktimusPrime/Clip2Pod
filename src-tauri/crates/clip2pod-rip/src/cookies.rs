// Browser cookies handed over by the extension: rendered as a Netscape cookie
// file for yt-dlp's `--cookies`, and the flag-stripping that lets them override
// whatever cookie flags the user's saved template carries.

use serde::Deserialize;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// One cookie as reported by `chrome.cookies.getAll`.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub domain: String,
    pub host_only: bool,
    pub path: String,
    pub secure: bool,
    #[serde(default)]
    pub expiration_date: Option<f64>,
    pub name: String,
    pub value: String,
}

// Manual Debug: the value is a credential and must never reach logs or panics.
impl std::fmt::Debug for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cookie")
            .field("domain", &self.domain)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// Cookies attached to a queued job. Debug shows only the count.
#[derive(Clone, Default)]
pub struct CookieJar(pub Vec<Cookie>);

impl std::fmt::Debug for CookieJar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CookieJar({} cookies)", self.0.len())
    }
}

fn bool_field(b: bool) -> &'static str {
    if b {
        "TRUE"
    } else {
        "FALSE"
    }
}

/// Render cookies as a Netscape cookie file. Cookies whose fields contain tab,
/// CR or LF would corrupt the format and are skipped.
pub fn netscape(cookies: &[Cookie]) -> String {
    let mut out = String::from("# Netscape HTTP Cookie File\n");
    for c in cookies {
        let bad = |s: &str| s.contains(['\t', '\n', '\r']);
        if [&c.domain, &c.path, &c.name, &c.value].into_iter().any(|s| bad(s)) {
            continue;
        }
        let expiry = c.expiration_date.map_or(0, |e| e as i64);
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            c.domain,
            bool_field(!c.host_only),
            c.path,
            bool_field(c.secure),
            expiry,
            c.name,
            c.value
        );
    }
    out
}

/// Drop every yt-dlp cookie flag (and the value of the value-taking ones).
pub fn strip_cookie_flags(args: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--cookies" | "--cookies-from-browser" => {
                it.next();
            }
            "--no-cookies" | "--no-cookies-from-browser" => {}
            _ if a.starts_with("--cookies=") || a.starts_with("--cookies-from-browser=") => {}
            _ => out.push(a),
        }
    }
    out
}

/// A temp cookie file that is deleted when dropped.
pub struct CookieFile {
    path: PathBuf,
}

impl CookieFile {
    pub fn write(cookies: &[Cookie]) -> std::io::Result<CookieFile> {
        let path = std::env::temp_dir().join(format!("clip2pod-cookies-{}.txt", uuid::Uuid::new_v4()));
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut file = opts.open(&path)?;
        // Construct the guard first so a failed write still removes the file.
        let guard = CookieFile { path };
        file.write_all(netscape(cookies).as_bytes())?;
        Ok(guard)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for CookieFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookie(domain: &str, host_only: bool, exp: Option<f64>, name: &str, value: &str) -> Cookie {
        Cookie {
            domain: domain.into(),
            host_only,
            path: "/".into(),
            secure: true,
            expiration_date: exp,
            name: name.into(),
            value: value.into(),
        }
    }

    #[test]
    fn netscape_formats_host_only_and_domain_cookies() {
        let out = netscape(&[
            cookie("www.reddit.com", true, Some(1767225600.7), "a", "1"),
            cookie(".reddit.com", false, None, "reddit_session", "x"),
        ]);
        assert!(out.starts_with("# Netscape HTTP Cookie File\n"));
        assert!(out.contains("www.reddit.com\tFALSE\t/\tTRUE\t1767225600\ta\t1\n"));
        assert!(out.contains(".reddit.com\tTRUE\t/\tTRUE\t0\treddit_session\tx\n"));
    }

    #[test]
    fn netscape_skips_values_with_tabs_or_newlines() {
        let out = netscape(&[
            cookie("a.com", true, None, "t", "a\tb"),
            cookie("a.com", true, None, "n", "a\nb"),
            cookie("a.com", true, None, "r", "a\rb"),
            cookie("a.com", true, None, "ok", "fine"),
        ]);
        assert!(!out.contains("a\tb") && !out.contains("a\nb") && !out.contains("a\rb"));
        assert!(out.contains("\tok\tfine\n"));
        assert_eq!(out.lines().count(), 2);
    }

    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn strip_cookie_flags_removes_every_form() {
        let input = v(&[
            "-x",
            "--cookies",
            "/c.txt",
            "--cookies-from-browser",
            "brave",
            "--cookies=/d.txt",
            "--cookies-from-browser=firefox",
            "--no-cookies",
            "--no-cookies-from-browser",
            "-o",
            "%(title)s.%(ext)s",
        ]);
        assert_eq!(strip_cookie_flags(input), v(&["-x", "-o", "%(title)s.%(ext)s"]));
        assert!(strip_cookie_flags(v(&["--cookies"])).is_empty());
    }

    #[test]
    fn cookie_file_written_then_deleted_on_drop() {
        let cookies = [cookie("a.com", true, None, "n", "v")];
        let f = CookieFile::write(&cookies).unwrap();
        let path = f.path().to_path_buf();
        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), netscape(&cookies));
        drop(f);
        assert!(!path.exists());
    }

    #[test]
    fn cookie_debug_hides_value() {
        let c = cookie("a.com", true, None, "sid", "SECRET123");
        let s = format!("{c:?}");
        assert!(!s.contains("SECRET123"));
        assert!(s.contains("a.com") && s.contains("sid"));
    }
}
