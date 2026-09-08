#[cfg(test)]
mod tests {
    use super::build_open_command;
    use std::ffi::OsStr;

    #[test]
    fn builds_xdg_open_command_for_target() {
        let cmd = build_open_command(OsStr::new("https://example.com"));
        assert_eq!(cmd.get_program(), OsStr::new("xdg-open"));
        assert_eq!(cmd.get_args().collect::<Vec<_>>(), vec![OsStr::new("https://example.com")]);
    }
}
