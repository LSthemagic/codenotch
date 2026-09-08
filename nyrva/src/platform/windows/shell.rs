use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn open_folder(path: &Path) {
    let mut cmd = Command::new("explorer");
    cmd.arg(path.as_os_str());
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(CREATE_NO_WINDOW);
    let _ = cmd.spawn();
}

pub fn open_url(url: &str) {
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "start", "", url]);
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(CREATE_NO_WINDOW);
    let _ = cmd.spawn();
}
