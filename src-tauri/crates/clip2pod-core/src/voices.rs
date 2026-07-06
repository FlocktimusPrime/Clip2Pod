use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AuthorGender {
    #[default]
    Unknown,
    Male,
    Female,
}

/// Edge TTS voice metadata, cached in config for offline launches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub name: String,
    pub short_name: String,
    pub gender: Gender,
    pub locale: String,
    pub language: String,
    pub country: String,
    pub category: String,
}

/// Round-robin cycling positions and gender alternation, persisted across
/// restarts so variety continues instead of resetting every launch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CyclingState {
    pub last_gender: Option<Gender>,
    pub male_idx: usize,
    pub female_idx: usize,
}

/// Keep English voices suited to narration: drop anything tagged "cartoon".
pub fn filter_narration_voices(all: Vec<VoiceInfo>) -> Vec<VoiceInfo> {
    all.into_iter()
        .filter(|v| v.locale.starts_with("en-") && !v.category.to_lowercase().contains("cartoon"))
        .collect()
}

/// Out-of-the-box enabled set: standard en-US voices, excluding the
/// "Multilingual" variants (duplicates of the standard voice).
pub fn default_enabled(voices: &[VoiceInfo]) -> HashSet<String> {
    voices
        .iter()
        .filter(|v| v.locale == "en-US" && !v.short_name.contains("Multilingual"))
        .map(|v| v.short_name.clone())
        .collect()
}

fn pick_from_pool(
    voices: &[VoiceInfo],
    enabled: &HashSet<String>,
    gender: Gender,
    state: &mut CyclingState,
) -> Option<VoiceInfo> {
    let mut pool: Vec<&VoiceInfo> = voices
        .iter()
        .filter(|v| v.gender == gender && enabled.contains(&v.short_name))
        .collect();
    pool.sort_by(|a, b| a.short_name.cmp(&b.short_name));
    if pool.is_empty() {
        return None;
    }
    let idx = match gender {
        Gender::Male => &mut state.male_idx,
        Gender::Female => &mut state.female_idx,
    };
    let voice = pool[*idx % pool.len()].clone();
    *idx = (*idx + 1) % pool.len();
    state.last_gender = Some(gender);
    Some(voice)
}

/// Pick the next voice honoring the author-gender preference; Unknown
/// alternates gender per generation, and each gender pool rotates
/// round-robin through its enabled voices. Advances `state`.
pub fn pick_voice(
    voices: &[VoiceInfo],
    enabled: &HashSet<String>,
    author_gender: AuthorGender,
    state: &mut CyclingState,
) -> Option<VoiceInfo> {
    match author_gender {
        AuthorGender::Male => pick_from_pool(voices, enabled, Gender::Male, state),
        AuthorGender::Female => pick_from_pool(voices, enabled, Gender::Female, state),
        AuthorGender::Unknown => {
            let target = match state.last_gender {
                Some(Gender::Male) => Gender::Female,
                Some(Gender::Female) => Gender::Male,
                None => Gender::Male,
            };
            let other = match target {
                Gender::Male => Gender::Female,
                Gender::Female => Gender::Male,
            };
            pick_from_pool(voices, enabled, target, state)
                .or_else(|| pick_from_pool(voices, enabled, other, state))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(short: &str, gender: Gender, locale: &str, category: &str) -> VoiceInfo {
        VoiceInfo {
            name: format!("Microsoft {short}"),
            short_name: short.to_string(),
            gender,
            locale: locale.to_string(),
            language: "en".to_string(),
            country: locale.split('-').nth(1).unwrap_or("").to_string(),
            category: category.to_string(),
        }
    }

    fn pool() -> Vec<VoiceInfo> {
        vec![
            v("en-US-GuyNeural", Gender::Male, "en-US", "General"),
            v("en-US-DavisNeural", Gender::Male, "en-US", "General"),
            v("en-US-JennyNeural", Gender::Female, "en-US", "General"),
            v("en-US-AriaNeural", Gender::Female, "en-US", "General"),
            v("en-GB-RyanNeural", Gender::Male, "en-GB", "General"),
        ]
    }

    fn all_enabled(voices: &[VoiceInfo]) -> HashSet<String> {
        voices.iter().map(|v| v.short_name.clone()).collect()
    }

    #[test]
    fn filter_drops_cartoon_voices() {
        let mut voices = pool();
        voices.push(v("en-US-CartoonyNeural", Gender::Male, "en-US", "Cartoon"));
        let filtered = filter_narration_voices(voices);
        assert_eq!(filtered.len(), 5);
        assert!(filtered.iter().all(|v| !v.category.eq_ignore_ascii_case("cartoon")));
    }

    #[test]
    fn default_enabled_is_standard_en_us_without_multilingual() {
        let mut voices = pool();
        voices.push(v("en-US-EmmaMultilingualNeural", Gender::Female, "en-US", "General"));
        let def = default_enabled(&voices);
        assert!(def.contains("en-US-GuyNeural"));
        assert!(def.contains("en-US-JennyNeural"));
        assert!(!def.contains("en-GB-RyanNeural"));
        assert!(!def.contains("en-US-EmmaMultilingualNeural"));
    }

    #[test]
    fn unknown_alternates_gender_each_pick() {
        let voices = pool();
        let enabled = all_enabled(&voices);
        let mut state = CyclingState::default();
        let g1 = pick_voice(&voices, &enabled, AuthorGender::Unknown, &mut state).unwrap().gender;
        let g2 = pick_voice(&voices, &enabled, AuthorGender::Unknown, &mut state).unwrap().gender;
        let g3 = pick_voice(&voices, &enabled, AuthorGender::Unknown, &mut state).unwrap().gender;
        assert_ne!(g1, g2);
        assert_eq!(g1, g3);
    }

    #[test]
    fn explicit_gender_uses_only_that_pool_and_rotates() {
        let voices = pool();
        let enabled = all_enabled(&voices);
        let mut state = CyclingState::default();
        let mut seen = Vec::new();
        for _ in 0..3 {
            let p = pick_voice(&voices, &enabled, AuthorGender::Male, &mut state).unwrap();
            assert_eq!(p.gender, Gender::Male);
            seen.push(p.short_name);
        }
        // 3 male voices enabled: all distinct before any repeat
        let unique: HashSet<_> = seen.iter().collect();
        assert_eq!(unique.len(), 3);
        // 4th pick wraps to the first again
        let p4 = pick_voice(&voices, &enabled, AuthorGender::Male, &mut state).unwrap();
        assert_eq!(p4.short_name, seen[0]);
    }

    #[test]
    fn cycling_continues_after_state_roundtrip() {
        let voices = pool();
        let enabled = all_enabled(&voices);
        let mut state = CyclingState::default();
        let first = pick_voice(&voices, &enabled, AuthorGender::Female, &mut state).unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let mut restored: CyclingState = serde_json::from_str(&json).unwrap();
        let second = pick_voice(&voices, &enabled, AuthorGender::Female, &mut restored).unwrap();
        assert_ne!(first.short_name, second.short_name);
    }

    #[test]
    fn skips_disabled_and_handles_empty_pool() {
        let voices = pool();
        let mut enabled = all_enabled(&voices);
        enabled.retain(|s| !voices.iter().any(|v| v.short_name == *s && v.gender == Gender::Female));
        let mut state = CyclingState::default();
        // Female pool empty: explicit Female yields None, Unknown falls back to male
        assert!(pick_voice(&voices, &enabled, AuthorGender::Female, &mut state).is_none());
        let p = pick_voice(&voices, &enabled, AuthorGender::Unknown, &mut state).unwrap();
        assert_eq!(p.gender, Gender::Male);
        assert!(pick_voice(&voices, &HashSet::new(), AuthorGender::Unknown, &mut state).is_none());
    }
}
