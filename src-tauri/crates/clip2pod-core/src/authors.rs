use crate::config::write_atomic;
use crate::voices::AuthorGender;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

/// A remembered author and the gender last chosen for their articles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorEntry {
    pub name: String,
    pub gender: AuthorGender,
}

/// Load `dir/authors.json`; missing or corrupt file yields an empty list.
pub fn load_authors(dir: &Path) -> Vec<AuthorEntry> {
    std::fs::read_to_string(dir.join("authors.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_authors(dir: &Path, authors: &[AuthorEntry]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(authors).map_err(io::Error::other)?;
    write_atomic(&dir.join("authors.json"), &json)
}

/// Exact (trimmed) match against a remembered author's name.
pub fn find(authors: &[AuthorEntry], name: &str) -> Option<AuthorGender> {
    let name = name.trim();
    authors.iter().find(|a| a.name == name).map(|a| a.gender)
}

/// Insert or overwrite. A brand-new name is only remembered the first time a
/// real gender (not Unknown) is picked for it — there's nothing worth
/// persisting otherwise. An existing entry is always overwritten, Unknown
/// included, so un-recognizing an author is just picking Unknown again.
/// Returns the gender now on record, or None if nothing was saved.
pub fn upsert(authors: &mut Vec<AuthorEntry>, name: &str, gender: AuthorGender) -> Option<AuthorGender> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    match authors.iter_mut().find(|a| a.name == name) {
        Some(entry) => {
            entry.gender = gender;
            Some(gender)
        }
        None if gender != AuthorGender::Unknown => {
            authors.push(AuthorEntry { name: name.to_string(), gender });
            Some(gender)
        }
        None => None,
    }
}

pub fn rename(authors: &mut [AuthorEntry], old: &str, new: &str) {
    let new = new.trim();
    if new.is_empty() {
        return;
    }
    if let Some(entry) = authors.iter_mut().find(|a| a.name == old) {
        entry.name = new.to_string();
    }
}

pub fn delete(authors: &mut Vec<AuthorEntry>, name: &str) {
    authors.retain(|a| a.name != name);
}

/// Fold `other` into `primary`: `other`'s row is dropped, `primary` is kept
/// exactly as it already is.
pub fn merge(authors: &mut Vec<AuthorEntry>, primary: &str, other: &str) {
    if primary == other {
        return;
    }
    authors.retain(|a| a.name != other);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_save_load() {
        let dir = tempdir().unwrap();
        let authors = vec![AuthorEntry { name: "Jane Doe".into(), gender: AuthorGender::Female }];
        save_authors(dir.path(), &authors).unwrap();
        let loaded = load_authors(dir.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Jane Doe");
        assert_eq!(loaded[0].gender, AuthorGender::Female);
    }

    #[test]
    fn missing_file_yields_empty() {
        let dir = tempdir().unwrap();
        assert!(load_authors(dir.path()).is_empty());
    }

    #[test]
    fn upsert_skips_creating_on_unknown_but_overwrites_existing() {
        let mut authors = Vec::new();
        assert_eq!(upsert(&mut authors, "Jane Doe", AuthorGender::Unknown), None);
        assert!(authors.is_empty());

        assert_eq!(upsert(&mut authors, "Jane Doe", AuthorGender::Female), Some(AuthorGender::Female));
        assert_eq!(authors.len(), 1);

        assert_eq!(upsert(&mut authors, "Jane Doe", AuthorGender::Unknown), Some(AuthorGender::Unknown));
        assert_eq!(find(&authors, "Jane Doe"), Some(AuthorGender::Unknown));
    }

    #[test]
    fn find_is_exact_and_trims_whitespace() {
        let authors = vec![AuthorEntry { name: "Jane Doe".into(), gender: AuthorGender::Female }];
        assert_eq!(find(&authors, "  Jane Doe  "), Some(AuthorGender::Female));
        assert_eq!(find(&authors, "jane doe"), None);
    }

    #[test]
    fn merge_drops_other_keeps_primary() {
        let mut authors = vec![
            AuthorEntry { name: "Jane Doe".into(), gender: AuthorGender::Female },
            AuthorEntry { name: "J. Doe".into(), gender: AuthorGender::Male },
        ];
        merge(&mut authors, "Jane Doe", "J. Doe");
        assert_eq!(authors.len(), 1);
        assert_eq!(find(&authors, "Jane Doe"), Some(AuthorGender::Female));
        assert_eq!(find(&authors, "J. Doe"), None);
    }

    #[test]
    fn rename_and_delete() {
        let mut authors = vec![AuthorEntry { name: "Jane Doe".into(), gender: AuthorGender::Female }];
        rename(&mut authors, "Jane Doe", "J. Doe");
        assert_eq!(find(&authors, "J. Doe"), Some(AuthorGender::Female));
        delete(&mut authors, "J. Doe");
        assert!(authors.is_empty());
    }
}
