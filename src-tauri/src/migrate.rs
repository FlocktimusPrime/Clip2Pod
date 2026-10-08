// One-time imports from older installs.
//
// Before 0.9 the config folder was `<config dir>/Clip2Pod2`. It is moved
// wholesale to `<config dir>/Clip2Pod`, so settings, logs and the feed token
// (and with it every subscribed phone) carry over.
//
// yt-dlFeed shipped as its own app with config at
// `<config dir>/yt-dlFeed/config.json`; the merged app keeps rip settings under
// `<config dir>/Clip2Pod/rip/`. On first run we copy the two rip-relevant
// fields across so a prior yt-dlFeed user keeps their episode folder and yt-dlp
// args. Narrate settings (theme, startup) are never sourced from here.

use std::path::Path;

/// Move the pre-0.9 `Clip2Pod2` folder next to `config_dir` into its place.
/// Must run before anything reads or creates `config_dir`.
pub fn migrate_legacy_config_dir(config_dir: &Path) {
    if let Some(parent) = config_dir.parent() {
        move_dir_if_absent(&parent.join("Clip2Pod2"), config_dir);
    }
}

/// Rename `old` to `new` unless `new` already exists (then both are left
/// alone) or there is no `old`.
fn move_dir_if_absent(old: &Path, new: &Path) {
    if new.exists() || !old.is_dir() {
        return;
    }
    match std::fs::rename(old, new) {
        Ok(()) => eprintln!(
            "clip2pod: moved settings from {} to {}",
            old.display(),
            new.display()
        ),
        Err(e) => eprintln!("clip2pod: could not move {}: {e}", old.display()),
    }
}

/// Import yt-dlFeed's config into `rip_dir` if it hasn't been imported yet.
pub fn migrate_ytdlfeed_config(rip_dir: &Path) {
    migrate_from(&ytdlfeed_core::config::default_config_dir(), rip_dir);
}

/// `rip_dir/config.json` existing is the "already migrated" flag. No-op when
/// there's nothing to import or the import already ran.
fn migrate_from(legacy_dir: &Path, rip_dir: &Path) {
    if rip_dir.join("config.json").exists() {
        return;
    }
    if !legacy_dir.join("config.json").exists() {
        return;
    }
    let old = ytdlfeed_core::config::load_config(legacy_dir);
    let migrated = ytdlfeed_core::config::Config {
        output_dir: old.output_dir,
        args_template: old.args_template,
        ytdlp_path: old.ytdlp_path,
        ..Default::default()
    };
    match ytdlfeed_core::config::save_config(rip_dir, &migrated) {
        Ok(()) => eprintln!(
            "clip2pod: imported yt-dlFeed settings from {}",
            legacy_dir.display()
        ),
        Err(e) => eprintln!("clip2pod: could not write migrated rip config: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_legacy(dir: &Path, json: &str) {
        std::fs::write(dir.join("config.json"), json).unwrap();
    }

    #[test]
    fn imports_rip_fields_but_not_narrate_ones() {
        let legacy = tempdir().unwrap();
        let rip = tempdir().unwrap();
        write_legacy(
            legacy.path(),
            r#"{"output_dir":"/media/pods","args_template":"--extract-audio","ytdlp_path":"/opt/yt-dlp","theme":"light","start_minimized":true}"#,
        );
        migrate_from(legacy.path(), rip.path());

        let got = ytdlfeed_core::config::load_config(rip.path());
        assert_eq!(got.output_dir.as_deref(), Some(Path::new("/media/pods")));
        assert_eq!(got.args_template.as_deref(), Some("--extract-audio"));
        assert_eq!(got.ytdlp_path.as_deref(), Some("/opt/yt-dlp"));
        // narrate-owned fields are left at defaults, not carried over
        assert_eq!(got.theme, ytdlfeed_core::config::Theme::Dark);
        assert!(!got.start_minimized);
    }

    #[test]
    fn no_op_when_already_migrated() {
        let legacy = tempdir().unwrap();
        let rip = tempdir().unwrap();
        write_legacy(legacy.path(), r#"{"output_dir":"/media/new"}"#);
        std::fs::write(rip.path().join("config.json"), r#"{"output_dir":"/media/kept"}"#).unwrap();

        migrate_from(legacy.path(), rip.path());

        let got = ytdlfeed_core::config::load_config(rip.path());
        assert_eq!(got.output_dir.as_deref(), Some(Path::new("/media/kept")));
    }

    #[test]
    fn moves_old_config_folder_with_its_contents() {
        let root = tempdir().unwrap();
        let old = root.path().join("Clip2Pod2");
        std::fs::create_dir_all(old.join("rip")).unwrap();
        std::fs::write(old.join("feed_token"), "abc").unwrap();
        std::fs::write(old.join("rip").join("config.json"), "{}").unwrap();
        let new = root.path().join("Clip2Pod");

        move_dir_if_absent(&old, &new);

        assert!(!old.exists());
        assert_eq!(std::fs::read_to_string(new.join("feed_token")).unwrap(), "abc");
        assert!(new.join("rip").join("config.json").exists());
    }

    #[test]
    fn leaves_both_folders_alone_when_new_one_exists() {
        let root = tempdir().unwrap();
        let old = root.path().join("Clip2Pod2");
        let new = root.path().join("Clip2Pod");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join("feed_token"), "old").unwrap();
        std::fs::write(new.join("feed_token"), "new").unwrap();

        move_dir_if_absent(&old, &new);

        assert_eq!(std::fs::read_to_string(old.join("feed_token")).unwrap(), "old");
        assert_eq!(std::fs::read_to_string(new.join("feed_token")).unwrap(), "new");
    }

    #[test]
    fn no_old_folder_creates_nothing() {
        let root = tempdir().unwrap();
        let new = root.path().join("Clip2Pod");
        move_dir_if_absent(&root.path().join("Clip2Pod2"), &new);
        assert!(!new.exists());
    }

    #[test]
    fn no_op_when_no_legacy_install() {
        let rip = tempdir().unwrap();
        migrate_from(&PathBuf::from("/definitely/not/here"), rip.path());
        assert!(!rip.path().join("config.json").exists());
    }
}
