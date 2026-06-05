use crate::{
    model::{AgentLocaleState, LOCALE_VERSION, MANUAL_LANGUAGE},
    templates::LOCALE_RELATIVE_PATH,
};
use std::{env, fs, path::Path};

pub(crate) fn expected_locale_state(
    root: &Path,
    app_locale: Option<&str>,
    checked_at: u64,
) -> AgentLocaleState {
    let mut warnings = Vec::new();

    if let Some(locale) = app_locale.and_then(normalize_locale) {
        return AgentLocaleState {
            version: LOCALE_VERSION.to_string(),
            raw_os_locale: app_locale.map(str::to_string),
            agent_locale: locale,
            manual_language: MANUAL_LANGUAGE.to_string(),
            source: "app".to_string(),
            checked_at,
            fallback: false,
            warnings,
        };
    }

    if let Some(raw_os_locale) = detect_os_locale() {
        if let Some(locale) = normalize_locale(&raw_os_locale) {
            return AgentLocaleState {
                version: LOCALE_VERSION.to_string(),
                raw_os_locale: Some(raw_os_locale),
                agent_locale: locale,
                manual_language: MANUAL_LANGUAGE.to_string(),
                source: "os".to_string(),
                checked_at,
                fallback: false,
                warnings,
            };
        }
        warnings.push(format!(
            "OS locale `{raw_os_locale}` could not be normalized to a BCP 47 locale."
        ));
    }

    if let Some(existing) = read_locale_state(root) {
        if normalize_locale(&existing.agent_locale).is_some()
            && existing.manual_language == MANUAL_LANGUAGE
        {
            return AgentLocaleState {
                checked_at,
                source: "existing-workspace".to_string(),
                ..existing
            };
        }
        warnings.push("Existing workspace locale state is invalid.".to_string());
    }

    warnings.push("OS / App locale unavailable; falling back to en-US.".to_string());
    AgentLocaleState {
        version: LOCALE_VERSION.to_string(),
        agent_locale: "en-US".to_string(),
        raw_os_locale: None,
        manual_language: MANUAL_LANGUAGE.to_string(),
        source: "fallback".to_string(),
        checked_at,
        fallback: true,
        warnings,
    }
}

pub(crate) fn read_locale_state(root: &Path) -> Option<AgentLocaleState> {
    let raw = fs::read_to_string(root.join(LOCALE_RELATIVE_PATH)).ok()?;
    serde_json::from_str(&raw).ok()
}

pub(crate) fn normalize_locale(raw: &str) -> Option<String> {
    let cleaned = raw
        .trim()
        .split(':')
        .next()
        .unwrap_or(raw)
        .split('.')
        .next()
        .unwrap_or(raw)
        .split('@')
        .next()
        .unwrap_or(raw)
        .replace('_', "-");
    let cleaned = cleaned.trim_matches('-').trim();
    if cleaned.is_empty() {
        return None;
    }
    let lower = cleaned.to_ascii_lowercase();
    if matches!(lower.as_str(), "c" | "posix" | "utf-8") {
        return None;
    }

    let parts = cleaned
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let language = parts.first()?.to_ascii_lowercase();
    if !is_language_subtag(&language) {
        return None;
    }

    if parts.len() == 1 {
        return Some(default_regional_locale(&language));
    }

    let mut normalized = vec![language];
    for part in parts.iter().skip(1) {
        normalized.push(normalize_subtag(part));
    }
    Some(normalized.join("-"))
}

fn detect_os_locale() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
        .into_iter()
        .filter_map(|key| env::var(key).ok())
        .find(|value| !value.trim().is_empty())
}

fn is_language_subtag(value: &str) -> bool {
    (2..=3).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_alphabetic())
}

fn normalize_subtag(value: &str) -> String {
    if value.len() == 2 && value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return value.to_ascii_uppercase();
    }
    if value.len() == 4 && value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        let mut chars = value.chars();
        let first = chars
            .next()
            .map(|ch| ch.to_ascii_uppercase())
            .unwrap_or_default();
        let rest = chars.as_str().to_ascii_lowercase();
        return format!("{first}{rest}");
    }
    value.to_ascii_lowercase()
}

fn default_regional_locale(language: &str) -> String {
    match language {
        "de" => "de-DE",
        "en" => "en-US",
        "es" => "es-ES",
        "fr" => "fr-FR",
        "it" => "it-IT",
        "ja" => "ja-JP",
        "ko" => "ko-KR",
        "pt" => "pt-BR",
        "ru" => "ru-RU",
        "zh" => "zh-CN",
        _ => return language.to_string(),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_locale;

    #[test]
    fn locale_normalization_keeps_bcp_47_shape() {
        assert_eq!(normalize_locale("zh_CN.UTF-8").as_deref(), Some("zh-CN"));
        assert_eq!(normalize_locale("en_us").as_deref(), Some("en-US"));
        assert_eq!(normalize_locale("pt_BR").as_deref(), Some("pt-BR"));
        assert_eq!(normalize_locale("ja").as_deref(), Some("ja-JP"));
        assert_eq!(normalize_locale("C"), None);
    }
}
