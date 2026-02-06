pub mod error;
pub mod logger;
pub mod os;
pub mod prompt;

pub use error::{
    config_error, package_error, symlink_error, ConfigErrorType,
    PackageErrorType, SymlinkErrorType,
};
pub use logger::{error, header, info, step, success, warning};
pub use os::{detect_os, os_name, LinuxDistro, OperatingSystem};
pub use prompt::{confirm, prompt};
