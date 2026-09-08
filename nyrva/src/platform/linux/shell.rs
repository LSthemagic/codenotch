use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

fn build_open_command(target: &OsStr) -> Command {
    let mut cmd = Command::new("xdg-open");
    cmd.arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd
}

fn open_target(target: &OsStr) {
    let _ = build_open_command(target).spawn();
}

pub fn open_folder(path: &Path) {
    open_target(path.as_os_str());
}

pub fn open_url(url: &str) {
    open_target(OsStr::new(url));
}

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
