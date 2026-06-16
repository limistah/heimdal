//! macOS defaults (preferences) sync functionality.

#[cfg(target_os = "macos")]
mod config;

#[cfg(target_os = "macos")]
pub use config::*;

/// Returns true if defaults sync is supported on this platform.
pub fn is_supported() -> bool {
    cfg!(target_os = "macos")
}
