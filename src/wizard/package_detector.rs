use anyhow::Result;
use std::process::Command;

/// Detector for installed packages on the system
pub struct PackageDetector;

/// A detected package
#[derive(Debug, Clone)]
pub struct DetectedPackage {
    pub name: String,
    pub manager: PackageManager,
    pub category: PackageCategory,
}

/// Package manager type
#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Homebrew,
    Apt,
    Dnf,
    Pacman,
    Mas, // Mac App Store
}

impl PackageManager {
    pub fn as_str(&self) -> &str {
        match self {
            PackageManager::Homebrew => "Homebrew",
            PackageManager::Apt => "APT",
            PackageManager::Dnf => "DNF",
            PackageManager::Pacman => "Pacman",
            PackageManager::Mas => "Mac App Store",
        }
    }
}

/// Package category for organization
#[derive(Debug, Clone, PartialEq)]
pub enum PackageCategory {
    Essential,   // git, vim, curl, etc.
    Development, // node, python, docker, etc.
    Terminal,    // tmux, fzf, ripgrep, etc.
    Editor,      // neovim, vscode, etc.
    Application, // browsers, productivity apps
    Other,
}

impl PackageCategory {
    pub fn as_str(&self) -> &str {
        match self {
            PackageCategory::Essential => "Essential",
            PackageCategory::Development => "Development",
            PackageCategory::Terminal => "Terminal Tools",
            PackageCategory::Editor => "Editors",
            PackageCategory::Application => "Applications",
            PackageCategory::Other => "Other",
        }
    }
}

impl PackageDetector {
    /// Detect all installed packages on the system
    pub fn detect_all() -> Result<Vec<DetectedPackage>> {
        let mut packages = Vec::new();

        // Detect which package managers are available and get their packages
        if Self::has_homebrew() {
            packages.extend(Self::detect_homebrew()?);
        }

        if Self::has_apt() {
            packages.extend(Self::detect_apt()?);
        }

        if Self::has_dnf() {
            packages.extend(Self::detect_dnf()?);
        }

        if Self::has_pacman() {
            packages.extend(Self::detect_pacman()?);
        }

        if Self::has_mas() {
            packages.extend(Self::detect_mas()?);
        }

        Ok(packages)
    }

    /// Check if Homebrew is available
    fn has_homebrew() -> bool {
        Self::command_exists("brew")
    }

    /// Check if APT is available
    fn has_apt() -> bool {
        Self::command_exists("apt")
    }

    /// Check if DNF is available
    fn has_dnf() -> bool {
        Self::command_exists("dnf")
    }

    /// Check if Pacman is available
    fn has_pacman() -> bool {
        Self::command_exists("pacman")
    }

    /// Check if mas (Mac App Store CLI) is available
    fn has_mas() -> bool {
        Self::command_exists("mas")
    }

    /// Check if a command exists in PATH
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Detect Homebrew packages
    fn detect_homebrew() -> Result<Vec<DetectedPackage>> {
        let mut packages = Vec::new();

        // Get formulae (CLI tools)
        let output = Command::new("brew").arg("list").arg("--formula").output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() {
                    packages.push(DetectedPackage {
                        name: name.clone(),
                        manager: PackageManager::Homebrew,
                        category: Self::categorize_package(&name),
                    });
                }
            }
        }

        // Get casks (GUI applications)
        let output = Command::new("brew").arg("list").arg("--cask").output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() {
                    packages.push(DetectedPackage {
                        name: format!("--cask {}", name),
                        manager: PackageManager::Homebrew,
                        category: PackageCategory::Application,
                    });
                }
            }
        }

        Ok(packages)
    }

    /// Detect APT packages (user-installed only)
    fn detect_apt() -> Result<Vec<DetectedPackage>> {
        let mut packages = Vec::new();

        // Get manually installed packages (not dependencies)
        let output = Command::new("apt-mark").arg("showmanual").output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() && Self::is_user_package(&name) {
                    packages.push(DetectedPackage {
                        name: name.clone(),
                        manager: PackageManager::Apt,
                        category: Self::categorize_package(&name),
                    });
                }
            }
        }

        Ok(packages)
    }

    /// Detect DNF packages
    fn detect_dnf() -> Result<Vec<DetectedPackage>> {
        let mut packages = Vec::new();

        let output = Command::new("dnf")
            .arg("list")
            .arg("--installed")
            .arg("--userinstalled")
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                // Skip header
                if let Some(name) = line.split_whitespace().next() {
                    let pkg_name = name.split('.').next().unwrap_or(name).to_string();
                    if Self::is_user_package(&pkg_name) {
                        packages.push(DetectedPackage {
                            name: pkg_name.clone(),
                            manager: PackageManager::Dnf,
                            category: Self::categorize_package(&pkg_name),
                        });
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Detect Pacman packages
    fn detect_pacman() -> Result<Vec<DetectedPackage>> {
        let mut packages = Vec::new();

        let output = Command::new("pacman")
            .arg("-Qe") // explicitly installed
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(name) = line.split_whitespace().next() {
                    let pkg_name = name.to_string();
                    if Self::is_user_package(&pkg_name) {
                        packages.push(DetectedPackage {
                            name: pkg_name.clone(),
                            manager: PackageManager::Pacman,
                            category: Self::categorize_package(&pkg_name),
                        });
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Detect Mac App Store applications
    fn detect_mas() -> Result<Vec<DetectedPackage>> {
        let mut packages = Vec::new();

        let output = Command::new("mas").arg("list").output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let app_name = parts[1].trim().to_string();
                    packages.push(DetectedPackage {
                        name: app_name,
                        manager: PackageManager::Mas,
                        category: PackageCategory::Application,
                    });
                }
            }
        }

        Ok(packages)
    }

    /// Check if a package is likely user-installed (not a system package)
    fn is_user_package(name: &str) -> bool {
        let system_packages = vec![
            "base",
            "systemd",
            "kernel",
            "glibc",
            "gcc-libs",
            "filesystem",
            "linux",
            "apt",
            "dpkg",
            "base-files",
            "libc",
            "coreutils",
        ];

        !system_packages.iter().any(|sys| name.contains(sys))
    }

    /// Categorize a package based on its name
    fn categorize_package(name: &str) -> PackageCategory {
        let lower = name.to_lowercase();

        // Check editors first (more specific patterns like neovim before vim)
        let editors = vec!["neovim", "nvim", "emacs", "vscode", "sublime", "code"];
        if editors.iter().any(|e| lower.contains(e)) {
            return PackageCategory::Editor;
        }

        // Essential tools
        let essential = vec!["git", "vim", "curl", "wget", "ssh", "rsync"];
        if essential.iter().any(|e| lower.contains(e)) {
            return PackageCategory::Essential;
        }

        // Development tools
        let development = vec![
            "node",
            "python",
            "ruby",
            "go",
            "rust",
            "cargo",
            "npm",
            "yarn",
            "docker",
            "kubectl",
            "terraform",
            "ansible",
            "java",
            "php",
            "composer",
            "pip",
        ];
        if development.iter().any(|d| lower.contains(d)) {
            return PackageCategory::Development;
        }

        // Terminal tools
        let terminal = vec![
            "tmux",
            "zsh",
            "fish",
            "starship",
            "fzf",
            "ripgrep",
            "bat",
            "exa",
            "fd",
            "htop",
            "btop",
            "alacritty",
            "kitty",
            "tree",
            "jq",
        ];
        if terminal.iter().any(|t| lower.contains(t)) {
            return PackageCategory::Terminal;
        }

        // If it's a cask or has certain patterns, it's likely an application
        if lower.contains("--cask") || lower.contains("app") {
            return PackageCategory::Application;
        }

        PackageCategory::Other
    }

    /// Group packages by category
    pub fn group_by_category(
        packages: &[DetectedPackage],
    ) -> Vec<(PackageCategory, Vec<&DetectedPackage>)> {
        let mut grouped: std::collections::HashMap<String, Vec<&DetectedPackage>> =
            std::collections::HashMap::new();

        for package in packages {
            grouped
                .entry(package.category.as_str().to_string())
                .or_insert_with(Vec::new)
                .push(package);
        }

        let mut result: Vec<(PackageCategory, Vec<&DetectedPackage>)> = Vec::new();

        // Order by category importance
        let categories = vec![
            PackageCategory::Essential,
            PackageCategory::Development,
            PackageCategory::Terminal,
            PackageCategory::Editor,
            PackageCategory::Application,
            PackageCategory::Other,
        ];

        for category in categories {
            if let Some(pkgs) = grouped.get(category.as_str()) {
                result.push((category.clone(), pkgs.clone()));
            }
        }

        result
    }

    /// Filter packages to only include commonly tracked ones
    pub fn filter_common(packages: Vec<DetectedPackage>) -> Vec<DetectedPackage> {
        packages
            .into_iter()
            .filter(|p| {
                matches!(
                    p.category,
                    PackageCategory::Essential
                        | PackageCategory::Development
                        | PackageCategory::Terminal
                        | PackageCategory::Editor
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_package() {
        assert_eq!(
            PackageDetector::categorize_package("git"),
            PackageCategory::Essential
        );
        assert_eq!(
            PackageDetector::categorize_package("python3"),
            PackageCategory::Development
        );
        assert_eq!(
            PackageDetector::categorize_package("tmux"),
            PackageCategory::Terminal
        );
        assert_eq!(
            PackageDetector::categorize_package("neovim"),
            PackageCategory::Editor
        );
    }

    #[test]
    fn test_is_user_package() {
        assert!(PackageDetector::is_user_package("git"));
        assert!(PackageDetector::is_user_package("neovim"));
        assert!(!PackageDetector::is_user_package("base-files"));
        assert!(!PackageDetector::is_user_package("systemd"));
    }

    #[test]
    fn test_command_exists() {
        // These should exist on most systems
        assert!(PackageDetector::command_exists("ls"));
        assert!(PackageDetector::command_exists("echo"));

        // This should not exist
        assert!(!PackageDetector::command_exists("nonexistent-command-xyz"));
    }
}
