//! Compatibility shim for shared callers.
//! The Windows implementation lives under `platform/windows/autostart.rs`.

pub use crate::platform::autostart::{disable, enable, is_enabled};
