//! macOS defaults (preferences) sync functionality.
//!
//! This module provides export, import, diff, and sync operations
//! for macOS preference domains stored as plist files.
//!
//! # Example Configuration
//!
//! ```yaml
//! defaults:
//!   enabled: true
//!   include:
//!     - com.apple.dock
//!     - com.apple.finder
//!   exclude:
//!     - com.apple.Safari.SandboxBroker
//!   path: macos-defaults  # relative to dotfiles dir
//! ```
// The re-exports below form the public library API. Not all of them are consumed
// by the binary itself, but they are available to downstream library users.
#![allow(unused_imports, dead_code)]

#[cfg(target_os = "macos")]
mod diff;
#[cfg(target_os = "macos")]
mod domains;
#[cfg(target_os = "macos")]
mod export;
#[cfg(target_os = "macos")]
mod import;
#[cfg(target_os = "macos")]
mod paths;
#[cfg(target_os = "macos")]
mod resolve;

#[cfg(target_os = "macos")]
pub use diff::{diff_all, diff_domain, DomainDiff, KeyDiff};
#[cfg(target_os = "macos")]
pub use domains::{list_filtered_domains, should_include_domain};
#[cfg(target_os = "macos")]
pub use export::{export_all, export_domains, plist_path_for_domain, ExportResult};
#[cfg(target_os = "macos")]
pub use import::{import_all, import_domains, ImportResult};
#[cfg(target_os = "macos")]
pub use paths::get_defaults_dir;
#[cfg(target_os = "macos")]
pub use resolve::{
    format_pref_value, print_domain_diff, prompt_resolution, resolve_all, Resolution,
};

/// Returns true if defaults sync is supported on this platform.
pub fn is_supported() -> bool {
    cfg!(target_os = "macos")
}
