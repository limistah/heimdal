use anyhow::Result;
use std::process::Command;

/// Common interface for all package managers
pub trait PackageManager: Send + Sync {
    /// Name of the package manager
    fn name(&self) -> &str;

    /// Check if this package manager is available on the system
    fn is_available(&self) -> bool;

    /// Check if a package is installed
    fn is_installed(&self, package: &str) -> bool;

    /// Install a package
    fn install(&self, package: &str, dry_run: bool) -> Result<()>;

    /// Install multiple packages at once (more efficient)
    fn install_many(&self, packages: &[String], dry_run: bool) -> Result<Vec<InstallResult>>;

    /// Update package manager's package list
    fn update(&self, dry_run: bool) -> Result<()>;
}

/// Result of a package installation attempt
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub package: String,
    pub success: bool,
    pub message: Option<String>,
    pub already_installed: bool,
}

impl InstallResult {
    pub fn success(package: String) -> Self {
        Self {
            package,
            success: true,
            message: None,
            already_installed: false,
        }
    }

    pub fn already_installed(package: String) -> Self {
        Self {
            package,
            success: true,
            message: Some("Already installed".to_string()),
            already_installed: true,
        }
    }

    pub fn failed(package: String, error: String) -> Self {
        Self {
            package,
            success: false,
            message: Some(error),
            already_installed: false,
        }
    }
}

/// Check if a command is available in PATH
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run a shell command and return its output
pub fn run_command(cmd: &str, args: &[&str], use_sudo: bool) -> Result<std::process::Output> {
    if use_sudo {
        let mut sudo_args = vec!["sudo", cmd];
        sudo_args.extend(args);
        Command::new("sh")
            .arg("-c")
            .arg(sudo_args.join(" "))
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run command: {}", e))
    } else {
        Command::new(cmd)
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run command: {}", e))
    }
}
