pub mod error;
pub mod logger;
pub mod os;
pub mod prompt;

pub use error::{
    config_error, format_error_with_help, package_error, symlink_error, ConfigErrorType,
    PackageErrorType, SymlinkErrorType,
};
pub use logger::{error, header, info, step, success, warning};
pub use os::{detect_os, is_linux, is_macos, os_name, LinuxDistro, OperatingSystem};
pub use prompt::{confirm, prompt};
