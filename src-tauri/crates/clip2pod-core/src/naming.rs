use std::path::Path;

/// Longest allowed filename stem (before ".mp3"), keeping paths comfortably
/// inside Windows' 260-char limit even in deep output directories.
const MAX_STEM_CHARS: usize = 80;

/// Make a title safe as a filename on Windows and Linux: strip characters
/// Windows forbids, collapse whitespace, trim trailing dots/spaces, cap length.
pub fn sanitize_filename(title: &str) -> String {
    const ILLEGAL: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let cleaned: String = title
        .chars()
        .filter(|c| !ILLEGAL.contains(c) && !c.is_control())
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let capped: String = collapsed.chars().take(MAX_STEM_CHARS).collect();
    let trimmed = capped.trim_end_matches(['.', ' ']).to_string();
    if trimmed.is_empty() {
        "Untitled".to_string()
    } else {
        trimmed
    }
}

/// Build the final output filename (with .mp3 extension), applying the
/// optional C2P_ prefix and appending " (2)", " (3)", ... until the name
/// collides with neither an existing file in `dir` nor a name in `reserved`
/// (filenames already claimed by queued/processing jobs).
pub fn output_filename(dir: &Path, filename_title: &str, prefix: bool, reserved: &[String]) -> String {
    let stem = sanitize_filename(filename_title);
    let stem = if prefix { format!("C2P_{stem}") } else { stem };
    let taken = |name: &str| dir.join(name).exists() || reserved.iter().any(|r| r == name);

    let candidate = format!("{stem}.mp3");
    if !taken(&candidate) {
        return candidate;
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{stem} ({n}).mp3");
        if !taken(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn strips_windows_illegal_characters() {
        assert_eq!(sanitize_filename(r#"a<b>c:d"e/f\g|h?i*j"#), "abcdefghij");
    }

    #[test]
    fn trims_trailing_dots_and_spaces() {
        assert_eq!(sanitize_filename("Story part 2... "), "Story part 2");
    }

    #[test]
    fn caps_length_and_collapses_whitespace() {
        let long = "word ".repeat(40);
        let s = sanitize_filename(&long);
        assert!(s.chars().count() <= 80);
        assert!(!s.contains("  "));
        assert_eq!(sanitize_filename("a\t b\n c"), "a b c");
    }

    #[test]
    fn empty_or_all_illegal_falls_back() {
        assert_eq!(sanitize_filename("???"), "Untitled");
        assert_eq!(sanitize_filename(""), "Untitled");
    }

    #[test]
    fn applies_prefix_and_extension() {
        let dir = tempdir().unwrap();
        assert_eq!(output_filename(dir.path(), "My Story", true, &[]), "C2P_My Story.mp3");
        assert_eq!(output_filename(dir.path(), "My Story", false, &[]), "My Story.mp3");
    }

    #[test]
    fn suffixes_on_disk_collision() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("Tale.mp3"), b"x").unwrap();
        assert_eq!(output_filename(dir.path(), "Tale", false, &[]), "Tale (2).mp3");
        std::fs::write(dir.path().join("Tale (2).mp3"), b"x").unwrap();
        assert_eq!(output_filename(dir.path(), "Tale", false, &[]), "Tale (3).mp3");
    }

    #[test]
    fn suffixes_on_reserved_queue_names() {
        let dir = tempdir().unwrap();
        let reserved = vec!["Tale.mp3".to_string(), "Tale (2).mp3".to_string()];
        assert_eq!(output_filename(dir.path(), "Tale", false, &reserved), "Tale (3).mp3");
    }
}
