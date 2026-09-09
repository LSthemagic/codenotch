pub mod autostart;
mod focus;
mod locale;
mod shell;
mod window;

pub use focus::{
    chain_of, fg_pid, focus_claude_desktop, focus_terminal, pid_hits_chain, proc_maps, ProcMaps,
};
pub use locale::system_language;
pub use shell::{open_folder, open_url};
pub use window::{apply_noactivate, attach_parent_console, left_button_down};
