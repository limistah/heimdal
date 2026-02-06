use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;
use crate::state::HeimdallState;

/// Status information for a dotfile
#[derive(Debug, Clone)]
pub struct DotfileStatus {
    pub source: PathBuf,
    pub target: PathBuf,
    pub status: LinkStatus,
}

/// Status of a symlink
#[derive(Debug, Clone, PartialEq)]
pub enum LinkStatus {
    /// Link exists and points to correct location
    Synced,
    /// Link exists but points to wrong location
    WrongTarget,
    /// Target exists but is not a symlink
    NotSymlink,
    /// Link doesn't exist
    Missing,
    /// Source file has been modified
    Modified,
}

/// Status information for a package
#[derive(Debug, Clone)]
pub struct PackageStatus {
    pub name: String,
    pub manager: String,
    pub status: InstallStatus,
    pub version: Option<String>,
}

/// Installation status of a package
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStatus {
    Installed,
    NotInstalled,
    Unknown,
}

/// Overall status information
#[derive(Debug)]
pub struct StatusInfo {
    pub profile: String,
    pub dotfiles_path: PathBuf,
    pub repo_url: String,
    pub last_sync: Option<String>,
    pub last_apply: Option<String>,
    pub git_branch: Option<String>,
    pub git_clean: bool,
    pub git_changes: Vec<String>,
    pub dotfiles: Vec<DotfileStatus>,
    pub packages: Vec<PackageStatus>,
    pub warnings: Vec<String>,
}

impl StatusInfo {
    /// Display status in a beautiful format
    pub fn display(&self, verbose: bool) {
        self.print_header();
        self.print_configuration();
        self.print_dotfiles(verbose);
        self.print_packages(verbose);
        self.print_git_status();
        self.print_warnings();
    }

    fn print_header(&self) {
        println!("{}", "Heimdal Status".bold().cyan());
        println!("{}", "━".repeat(80).bright_black());
        println!();
    }

    fn print_configuration(&self) {
        println!("{}", "Configuration".bold());
        println!("  {}  {}", "Profile:".bright_black(), self.profile.green());
        println!(
            "  {}    {}",
            "Dotfiles:".bright_black(),
            self.dotfiles_path.display().to_string().yellow()
        );

        // Format last sync time
        if let Some(last_sync) = &self.last_sync {
            let time_ago = format_time_ago(last_sync);
            println!("  {}   {}", "Last sync:".bright_black(), time_ago);
        } else {
            println!("  {}   {}", "Last sync:".bright_black(), "Never".red());
        }

        // Format last apply time
        if let Some(last_apply) = &self.last_apply {
            let time_ago = format_time_ago(last_apply);
            println!("  {}  {}", "Last apply:".bright_black(), time_ago);
        } else {
            println!("  {}  {}", "Last apply:".bright_black(), "Never".red());
        }

        println!();
    }

    fn print_dotfiles(&self, verbose: bool) {
        let total = self.dotfiles.len();
        let synced = self
            .dotfiles
            .iter()
            .filter(|d| d.status == LinkStatus::Synced)
            .count();
        let modified = self
            .dotfiles
            .iter()
            .filter(|d| d.status == LinkStatus::Modified)
            .count();
        let missing = self
            .dotfiles
            .iter()
            .filter(|d| d.status == LinkStatus::Missing)
            .count();

        let summary = if synced == total {
            format!("{} tracked", total).green()
        } else {
            format!("{} tracked, {} issues", total, total - synced).yellow()
        };

        println!("{} ({})", "Dotfiles".bold(), summary);

        if verbose || self.dotfiles.len() <= 10 {
            // Show all files
            for dotfile in &self.dotfiles {
                self.print_dotfile_status(dotfile);
            }
        } else {
            // Show first 5 files, then summary
            for dotfile in self.dotfiles.iter().take(5) {
                self.print_dotfile_status(dotfile);
            }
            let remaining = self.dotfiles.len() - 5;
            println!(
                "  {} ({}, run with --verbose)",
                format!("... {} more", remaining).bright_black(),
                format!(
                    "{} synced, {} modified, {} missing",
                    synced - 5.min(synced),
                    modified,
                    missing
                )
                .bright_black()
            );
        }

        println!();
    }

    fn print_dotfile_status(&self, dotfile: &DotfileStatus) {
        let (icon, status_text, color_fn): (&str, String, fn(String) -> ColoredString) =
            match dotfile.status {
                LinkStatus::Synced => ("✓", "synced".to_string(), |s| s.green()),
                LinkStatus::Modified => ("⚠", "modified".to_string(), |s| s.yellow()),
                LinkStatus::Missing => ("✗", "missing".to_string(), |s| s.red()),
                LinkStatus::WrongTarget => ("⚠", "wrong target".to_string(), |s| s.yellow()),
                LinkStatus::NotSymlink => ("⚠", "not symlink".to_string(), |s| s.yellow()),
            };

        let source_name = dotfile
            .source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");

        println!(
            "  {} {}  →  {}  ({})",
            icon,
            source_name.cyan(),
            dotfile.target.display().to_string().bright_black(),
            color_fn(status_text)
        );
    }

    fn print_packages(&self, verbose: bool) {
        let total = self.packages.len();
        let installed = self
            .packages
            .iter()
            .filter(|p| p.status == InstallStatus::Installed)
            .count();
        let not_installed = self
            .packages
            .iter()
            .filter(|p| p.status == InstallStatus::NotInstalled)
            .count();

        let summary = if not_installed == 0 {
            format!("{} installed", total).green()
        } else {
            format!("{} installed, {} pending", installed, not_installed).yellow()
        };

        println!("{} ({})", "Packages".bold(), summary);

        if verbose || self.packages.len() <= 10 {
            // Show all packages
            for package in &self.packages {
                self.print_package_status(package);
            }
        } else {
            // Show first 5 packages, then summary
            for package in self.packages.iter().take(5) {
                self.print_package_status(package);
            }
            let remaining = self.packages.len() - 5;
            println!(
                "  {} ({}, run with --verbose)",
                format!("... {} more", remaining).bright_black(),
                format!(
                    "{} installed, {} pending",
                    installed - 5.min(installed),
                    not_installed
                )
                .bright_black()
            );
        }

        println!();
    }

    fn print_package_status(&self, package: &PackageStatus) {
        let (icon, status_text, color_fn): (&str, String, fn(String) -> ColoredString) =
            match package.status {
                InstallStatus::Installed => ("✓", "installed".to_string(), |s| s.green()),
                InstallStatus::NotInstalled => ("✗", "not installed".to_string(), |s| s.red()),
                InstallStatus::Unknown => ("?", "unknown".to_string(), |s| s.bright_black()),
            };

        let version_str = if let Some(version) = &package.version {
            format!(" v{}", version)
        } else {
            String::new()
        };

        println!(
            "  {} {}  ({}){}  {}",
            icon,
            package.name.cyan(),
            package.manager.bright_black(),
            version_str.bright_black(),
            color_fn(status_text)
        );
    }

    fn print_git_status(&self) {
        if let Some(branch) = &self.git_branch {
            let branch_display = if self.git_clean {
                format!("{} (clean)", branch).green()
            } else {
                format!("{} ({} changes)", branch, self.git_changes.len()).yellow()
            };

            println!("{}", "Git Status".bold());
            println!("  {}    {}", "Branch:".bright_black(), branch_display);

            if !self.git_clean && !self.git_changes.is_empty() {
                println!("  {}   {}", "Changes:".bright_black(), "");
                for change in self.git_changes.iter().take(10) {
                    println!("    {}", change.yellow());
                }
                if self.git_changes.len() > 10 {
                    println!(
                        "    {}",
                        format!("... {} more", self.git_changes.len() - 10).bright_black()
                    );
                }
            }

            println!();
        }
    }

    fn print_warnings(&self) {
        if !self.warnings.is_empty() {
            println!("{}", "Warnings".bold().yellow());
            for warning in &self.warnings {
                println!("  {} {}", "⚠".yellow(), warning.yellow());
            }
            println!();
        }
    }
}

/// Run the status command
pub fn run_status(verbose: bool) -> Result<()> {
    // Load state
    let state = HeimdallState::load()?;

    // Gather status information
    let mut status_info = StatusInfo {
        profile: state.active_profile.clone(),
        dotfiles_path: state.dotfiles_path.clone(),
        repo_url: state.repo_url.clone(),
        last_sync: state.last_sync.clone(),
        last_apply: state.last_apply.clone(),
        git_branch: None,
        git_clean: true,
        git_changes: Vec::new(),
        dotfiles: Vec::new(),
        packages: Vec::new(),
        warnings: Vec::new(),
    };

    // Get git status
    if let Ok((branch, changes)) = get_git_status(&state.dotfiles_path) {
        status_info.git_branch = Some(branch);
        status_info.git_clean = changes.is_empty();
        status_info.git_changes = changes;
    }

    // Get dotfile status
    if let Ok(dotfiles) = get_dotfile_status(&state.dotfiles_path) {
        status_info.dotfiles = dotfiles;
    }

    // Get package status
    if let Ok(packages) = get_package_status(&state.dotfiles_path, &state.active_profile) {
        status_info.packages = packages;
    }

    // Generate warnings
    status_info.warnings = generate_warnings(&status_info);

    // Display status
    status_info.display(verbose);

    Ok(())
}

/// Get git status for the dotfiles repository
fn get_git_status(dotfiles_path: &Path) -> Result<(String, Vec<String>)> {
    // Get current branch
    let branch_output = Command::new("git")
        .arg("-C")
        .arg(dotfiles_path)
        .arg("branch")
        .arg("--show-current")
        .output()
        .context("Failed to execute git branch")?;

    let branch = if branch_output.status.success() {
        String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string()
    } else {
        "unknown".to_string()
    };

    // Get status
    let status_output = Command::new("git")
        .arg("-C")
        .arg(dotfiles_path)
        .arg("status")
        .arg("--short")
        .output()
        .context("Failed to execute git status")?;

    let changes = if status_output.status.success() {
        String::from_utf8_lossy(&status_output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };

    Ok((branch, changes))
}

/// Get status of dotfiles (symlinks)
fn get_dotfile_status(dotfiles_path: &Path) -> Result<Vec<DotfileStatus>> {
    let mut dotfiles = Vec::new();

    // For now, we'll do a simple scan of common dotfiles
    // In a real implementation, this would read from heimdal.yaml
    let common_dotfiles = vec![
        ".zshrc",
        ".bashrc",
        ".vimrc",
        ".tmux.conf",
        ".gitconfig",
        ".config/nvim/init.vim",
    ];

    let home_dir = dirs::home_dir().context("Failed to get home directory")?;

    for file in common_dotfiles {
        let source = dotfiles_path.join(file);
        let target = home_dir.join(file);

        if !source.exists() {
            continue;
        }

        let status = if !target.exists() {
            LinkStatus::Missing
        } else if target.is_symlink() {
            match std::fs::read_link(&target) {
                Ok(link_target) => {
                    if link_target == source {
                        LinkStatus::Synced
                    } else {
                        LinkStatus::WrongTarget
                    }
                }
                Err(_) => LinkStatus::Missing,
            }
        } else {
            LinkStatus::NotSymlink
        };

        dotfiles.push(DotfileStatus {
            source,
            target,
            status,
        });
    }

    Ok(dotfiles)
}

/// Get status of packages
fn get_package_status(dotfiles_path: &Path, profile_name: &str) -> Result<Vec<PackageStatus>> {
    let mut packages = Vec::new();

    // Try to load heimdal.yaml
    let config_path = dotfiles_path.join("heimdal.yaml");
    if !config_path.exists() {
        return Ok(packages);
    }

    // Load config
    let config = config::load_config(&config_path)?;
    let resolved = config::resolve_profile(&config, profile_name)?;

    // Check homebrew packages
    if let Some(homebrew) = &resolved.sources.homebrew {
        for package in &homebrew.packages {
            let status = check_homebrew_package(package);
            packages.push(PackageStatus {
                name: package.clone(),
                manager: "homebrew".to_string(),
                status: status.0,
                version: status.1,
            });
        }
    }

    // Limit to first 20 packages to avoid slowdown
    packages.truncate(20);

    Ok(packages)
}

/// Check if a Homebrew package is installed
fn check_homebrew_package(package: &str) -> (InstallStatus, Option<String>) {
    let output = Command::new("brew")
        .arg("list")
        .arg("--versions")
        .arg(package)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version = version_str.split_whitespace().nth(1).map(|s| s.to_string());
            (InstallStatus::Installed, version)
        }
        _ => (InstallStatus::NotInstalled, None),
    }
}

/// Generate warnings based on status
fn generate_warnings(status_info: &StatusInfo) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check for modified dotfiles
    let modified_count = status_info
        .dotfiles
        .iter()
        .filter(|d| d.status == LinkStatus::Modified)
        .count();
    if modified_count > 0 {
        warnings.push(format!("{} dotfile(s) modified locally", modified_count));
    }

    // Check for missing dotfiles
    let missing_count = status_info
        .dotfiles
        .iter()
        .filter(|d| d.status == LinkStatus::Missing)
        .count();
    if missing_count > 0 {
        warnings.push(format!("{} dotfile(s) not symlinked", missing_count));
    }

    // Check for not installed packages
    let not_installed_count = status_info
        .packages
        .iter()
        .filter(|p| p.status == InstallStatus::NotInstalled)
        .count();
    if not_installed_count > 0 {
        warnings.push(format!("{} package(s) not installed", not_installed_count));
    }

    // Check for git changes
    if !status_info.git_clean {
        warnings.push(format!(
            "{} uncommitted change(s) in repository",
            status_info.git_changes.len()
        ));
    }

    // Check for stale sync
    if let Some(last_sync) = &status_info.last_sync {
        if is_time_stale(last_sync, 7) {
            warnings.push("Last sync was more than 7 days ago".to_string());
        }
    } else {
        warnings.push("Repository has never been synced".to_string());
    }

    warnings
}

/// Format time ago string (e.g., "2 hours ago", "3 days ago")
fn format_time_ago(timestamp: &str) -> ColoredString {
    match chrono::DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(dt);

            let text = if duration.num_days() > 0 {
                format!("{} days ago", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{} hours ago", duration.num_hours())
            } else if duration.num_minutes() > 0 {
                format!("{} minutes ago", duration.num_minutes())
            } else {
                "just now".to_string()
            };

            // Color based on staleness
            if duration.num_days() > 7 {
                text.red()
            } else if duration.num_days() > 3 {
                text.yellow()
            } else {
                text.green()
            }
        }
        Err(_) => timestamp.to_string().bright_black(),
    }
}

/// Check if a timestamp is stale (older than N days)
fn is_time_stale(timestamp: &str, days: i64) -> bool {
    match chrono::DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(dt);
            duration.num_days() > days
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_status_display() {
        let status = LinkStatus::Synced;
        assert_eq!(status, LinkStatus::Synced);

        let status = LinkStatus::Modified;
        assert_eq!(status, LinkStatus::Modified);
    }

    #[test]
    fn test_install_status_display() {
        let status = InstallStatus::Installed;
        assert_eq!(status, InstallStatus::Installed);

        let status = InstallStatus::NotInstalled;
        assert_eq!(status, InstallStatus::NotInstalled);
    }

    #[test]
    fn test_format_time_ago() {
        // Test recent time
        let now = chrono::Utc::now();
        let timestamp = now.to_rfc3339();
        let formatted = format_time_ago(&timestamp);
        assert!(formatted.to_string().contains("just now"));

        // Test hours ago
        let hours_ago = now - chrono::Duration::hours(5);
        let timestamp = hours_ago.to_rfc3339();
        let formatted = format_time_ago(&timestamp);
        assert!(formatted.to_string().contains("hours ago"));

        // Test days ago
        let days_ago = now - chrono::Duration::days(3);
        let timestamp = days_ago.to_rfc3339();
        let formatted = format_time_ago(&timestamp);
        assert!(formatted.to_string().contains("days ago"));
    }

    #[test]
    fn test_is_time_stale() {
        let now = chrono::Utc::now();

        // Recent time should not be stale
        let recent = now.to_rfc3339();
        assert!(!is_time_stale(&recent, 7));

        // Old time should be stale
        let old = (now - chrono::Duration::days(10)).to_rfc3339();
        assert!(is_time_stale(&old, 7));
    }

    #[test]
    fn test_generate_warnings() {
        let status_info = StatusInfo {
            profile: "test".to_string(),
            dotfiles_path: PathBuf::from("/test"),
            repo_url: "test".to_string(),
            last_sync: None,
            last_apply: None,
            git_branch: Some("main".to_string()),
            git_clean: false,
            git_changes: vec!["M file1".to_string(), "A file2".to_string()],
            dotfiles: vec![
                DotfileStatus {
                    source: PathBuf::from("/test/.zshrc"),
                    target: PathBuf::from("/home/.zshrc"),
                    status: LinkStatus::Modified,
                },
                DotfileStatus {
                    source: PathBuf::from("/test/.bashrc"),
                    target: PathBuf::from("/home/.bashrc"),
                    status: LinkStatus::Missing,
                },
            ],
            packages: vec![
                PackageStatus {
                    name: "git".to_string(),
                    manager: "homebrew".to_string(),
                    status: InstallStatus::Installed,
                    version: Some("2.43.0".to_string()),
                },
                PackageStatus {
                    name: "docker".to_string(),
                    manager: "homebrew".to_string(),
                    status: InstallStatus::NotInstalled,
                    version: None,
                },
            ],
            warnings: Vec::new(),
        };

        let warnings = generate_warnings(&status_info);

        // Should have warnings for:
        // - 1 modified dotfile
        // - 1 missing dotfile
        // - 1 not installed package
        // - 2 git changes
        // - Never synced
        assert_eq!(warnings.len(), 5);
        assert!(warnings.iter().any(|w| w.contains("modified")));
        assert!(warnings.iter().any(|w| w.contains("not symlinked")));
        assert!(warnings.iter().any(|w| w.contains("not installed")));
        assert!(warnings.iter().any(|w| w.contains("uncommitted")));
        assert!(warnings.iter().any(|w| w.contains("never been synced")));
    }

    #[test]
    fn test_dotfile_status_creation() {
        let status = DotfileStatus {
            source: PathBuf::from("/dotfiles/.zshrc"),
            target: PathBuf::from("/home/user/.zshrc"),
            status: LinkStatus::Synced,
        };

        assert_eq!(status.status, LinkStatus::Synced);
        assert_eq!(status.source, PathBuf::from("/dotfiles/.zshrc"));
    }

    #[test]
    fn test_package_status_creation() {
        let status = PackageStatus {
            name: "git".to_string(),
            manager: "homebrew".to_string(),
            status: InstallStatus::Installed,
            version: Some("2.43.0".to_string()),
        };

        assert_eq!(status.name, "git");
        assert_eq!(status.status, InstallStatus::Installed);
        assert_eq!(status.version, Some("2.43.0".to_string()));
    }
}
