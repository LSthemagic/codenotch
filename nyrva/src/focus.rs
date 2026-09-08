//! Compatibility shim for existing shared callers.
//! The Windows implementation lives under `platform/windows/focus.rs`.

pub use crate::platform::{
    chain_of, fg_pid, focus_claude_desktop, focus_terminal, pid_hits_chain, proc_maps, ProcMaps,
};
