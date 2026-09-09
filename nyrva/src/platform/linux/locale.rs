fn normalize_locale(raw: &str) -> &'static str {
    let l = raw.to_ascii_lowercase();
    if l.starts_with("zh") {
        "zh"
    } else if l.starts_with("ja") {
        "ja"
    } else if l.starts_with("ko") {
        "ko"
    } else {
        "en"
    }
}

pub fn system_language() -> &'static str {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                return normalize_locale(&v);
            }
        }
    }
    "en"
}

#[cfg(test)]
mod tests {
    use super::normalize_locale;

    #[test]
    fn normalizes_supported_locales() {
        assert_eq!(normalize_locale("zh_CN.UTF-8"), "zh");
        assert_eq!(normalize_locale("ja_JP.UTF-8"), "ja");
        assert_eq!(normalize_locale("ko_KR.UTF-8"), "ko");
        assert_eq!(normalize_locale("pt_BR.UTF-8"), "en");
    }

    #[test]
    fn normalizes_empty_and_c_locale_to_english() {
        assert_eq!(normalize_locale(""), "en");
        assert_eq!(normalize_locale("C"), "en");
        assert_eq!(normalize_locale("C.UTF-8"), "en");
    }
}
