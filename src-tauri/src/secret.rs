// Random secrets persisted in the config dir: the LAN feed token (feed.rs) and
// the extension pairing code (capture.rs). Saved owner-only on Unix so other
// local users can't read them.

use std::path::Path;

/// 128 random bits as lowercase hex.
pub fn new_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("OS random source unavailable");
    to_hex(&bytes)
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn is_valid_token(s: &str) -> bool {
    s.len() == 32 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The token saved in `dir/file`, or a fresh one (saved) on first run or if the
/// file is missing or mangled. The bool is true when a valid token was already
/// there.
pub fn load_or_create(dir: &Path, file: &str) -> (String, bool) {
    if let Ok(saved) = std::fs::read_to_string(dir.join(file)) {
        if is_valid_token(saved.trim()) {
            return (saved.trim().to_string(), true);
        }
    }
    let token = new_token();
    if let Err(e) = save(dir, file, &token) {
        eprintln!("failed to save {file}: {e}");
    }
    (token, false)
}

pub fn save(dir: &Path, file: &str, token: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    std::fs::create_dir_all(dir)?;
    let path = dir.join(file);
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        opts.mode(0o600);
        // `mode` only applies on create; tighten a file left by older versions.
        if path.exists() {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
    }
    opts.open(&path)?.write_all(token.as_bytes())
}

/// Constant-time compare, so response timing doesn't leak how much of a
/// guessed token was right.
pub fn same_token(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0, |acc, (x, y)| acc | (x ^ y))
            == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_round_trips_and_bad_file_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let (first, existed) = load_or_create(dir.path(), "t");
        assert!(is_valid_token(&first) && !existed);
        assert_eq!(load_or_create(dir.path(), "t"), (first.clone(), true));

        std::fs::write(dir.path().join("t"), "garbage").unwrap();
        let (replaced, existed) = load_or_create(dir.path(), "t");
        assert!(is_valid_token(&replaced) && !existed);
        assert_ne!(replaced, first);
    }

    #[cfg(unix)]
    #[test]
    fn saved_token_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("t"), "old").unwrap(); // default perms
        save(dir.path(), "t", &new_token()).unwrap();
        let mode = std::fs::metadata(dir.path().join("t")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn same_token_compares_exactly() {
        assert!(same_token("abc", "abc"));
        assert!(!same_token("abc", "abd"));
        assert!(!same_token("abc", "abcd"));
    }
}
