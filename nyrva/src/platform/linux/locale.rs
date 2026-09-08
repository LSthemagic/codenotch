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
