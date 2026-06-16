//! macOS defaults (preferences) sync functionality.

#[cfg(target_os = "macos")]
mod paths;

#[cfg(target_os = "macos")]
pub use paths::*;

#[cfg(target_os = "macos")]
mod domains;

#[cfg(target_os = "macos")]
pub use domains::*;

/// Returns true if defaults sync is supported on this platform.
pub fn is_supported() -> bool {
    cfg!(target_os = "macos")
}
