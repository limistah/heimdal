use anyhow::Result;

#[allow(dead_code)]
pub struct InstallResult {
    pub package: String,
    pub success: bool,
    pub already_installed: bool,
    pub message: Option<String>,
}

pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn field_name(&self) -> &str; // matches PackageMap field: "homebrew", "apt", etc.
    fn is_available(&self) -> bool;
    #[allow(dead_code)]
    fn is_installed(&self, pkg: &str) -> bool;
    fn install_many(&self, pkgs: &[String], dry_run: bool) -> Result<Vec<InstallResult>>;
}

// ── Homebrew ──────────────────────────────────────────────────────────────────

pub struct Homebrew;

impl PackageManager for Homebrew {
    fn name(&self) -> &str {
        "homebrew"
    }
    fn field_name(&self) -> &str {
        "homebrew"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("brew")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, pkg: &str) -> bool {
        std::process::Command::new("brew")
            .args(["list", "--versions", pkg])
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false)
    }

    fn install_many(&self, pkgs: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        install_with_cmd("brew", &["install"], pkgs, dry_run)
    }
}

// ── Homebrew Cask ─────────────────────────────────────────────────────────────

pub struct HomebrewCask;

impl PackageManager for HomebrewCask {
    fn name(&self) -> &str {
        "homebrew-cask"
    }
    fn field_name(&self) -> &str {
        "homebrew_casks"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("brew")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, pkg: &str) -> bool {
        std::process::Command::new("brew")
            .args(["list", "--cask", "--versions", pkg])
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false)
    }

    fn install_many(&self, pkgs: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        install_with_cmd("brew", &["install", "--cask"], pkgs, dry_run)
    }
}

// ── Apt ───────────────────────────────────────────────────────────────────────

pub struct Apt;

impl PackageManager for Apt {
    fn name(&self) -> &str {
        "apt"
    }
    fn field_name(&self) -> &str {
        "apt"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("apt-get")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, pkg: &str) -> bool {
        std::process::Command::new("dpkg-query")
            .args(["-W", "-f=${Status}", pkg])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("install ok installed"))
            .unwrap_or(false)
    }

    fn install_many(&self, pkgs: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        install_with_cmd("apt-get", &["install", "-y"], pkgs, dry_run)
    }
}

// ── Dnf ───────────────────────────────────────────────────────────────────────

pub struct Dnf;

impl PackageManager for Dnf {
    fn name(&self) -> &str {
        "dnf"
    }
    fn field_name(&self) -> &str {
        "dnf"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("dnf")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, pkg: &str) -> bool {
        std::process::Command::new("rpm")
            .args(["-q", pkg])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_many(&self, pkgs: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        install_with_cmd("dnf", &["install", "-y"], pkgs, dry_run)
    }
}

// ── Pacman ────────────────────────────────────────────────────────────────────

pub struct Pacman;

impl PackageManager for Pacman {
    fn name(&self) -> &str {
        "pacman"
    }
    fn field_name(&self) -> &str {
        "pacman"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("pacman")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, pkg: &str) -> bool {
        std::process::Command::new("pacman")
            .args(["-Q", pkg])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_many(&self, pkgs: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        install_with_cmd("pacman", &["-S", "--noconfirm"], pkgs, dry_run)
    }
}

// ── Apk ───────────────────────────────────────────────────────────────────────

pub struct Apk;

impl PackageManager for Apk {
    fn name(&self) -> &str {
        "apk"
    }
    fn field_name(&self) -> &str {
        "apk"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("apk")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, pkg: &str) -> bool {
        std::process::Command::new("apk")
            .args(["info", "-e", pkg])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install_many(&self, pkgs: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        install_with_cmd("apk", &["add"], pkgs, dry_run)
    }
}

// ── Shared helper ─────────────────────────────────────────────────────────────

fn install_with_cmd(
    cmd: &str,
    base_args: &[&str],
    pkgs: &[String],
    dry_run: bool,
) -> Result<Vec<InstallResult>> {
    if pkgs.is_empty() {
        return Ok(vec![]);
    }

    if dry_run {
        return Ok(pkgs
            .iter()
            .map(|p| InstallResult {
                package: p.clone(),
                success: true,
                already_installed: false,
                message: Some(format!("[dry-run] Would install: {}", p)),
            })
            .collect());
    }

    let mut results = Vec::new();
    for pkg in pkgs {
        let mut command = std::process::Command::new(cmd);
        command.args(base_args).arg(pkg);
        let output = command
            .output()
            .map_err(|e| crate::error::HeimdallError::Package {
                manager: cmd.to_string(),
                reason: format!("Cannot run {}: {}", cmd, e),
            })?;
        results.push(InstallResult {
            success: output.status.success(),
            already_installed: false,
            message: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).trim().to_string())
            },
            package: pkg.clone(),
        });
    }
    Ok(results)
}

// ── Public interface ──────────────────────────────────────────────────────────

/// Detect the currently available package manager (first one that's available).
pub fn detect_manager() -> Option<Box<dyn PackageManager>> {
    let managers: Vec<Box<dyn PackageManager>> = vec![
        Box::new(Homebrew),
        Box::new(Apt),
        Box::new(Dnf),
        Box::new(Pacman),
        Box::new(Apk),
    ];
    managers.into_iter().find(|m| m.is_available())
}

/// Install packages for the active profile (called from apply command).
pub fn install_for_profile(profile: &crate::config::Profile, dry_run: bool) -> Result<()> {
    let managers: Vec<Box<dyn PackageManager>> = vec![
        Box::new(Homebrew),
        Box::new(HomebrewCask),
        Box::new(Apt),
        Box::new(Dnf),
        Box::new(Pacman),
        Box::new(Apk),
    ];

    let pkgs = &profile.packages;

    // Install common packages via the first available package manager
    if !pkgs.common.is_empty() {
        match managers.iter().find(|m| m.is_available()) {
            Some(manager) => {
                let results = manager.install_many(&pkgs.common, dry_run)?;
                for r in &results {
                    if r.success {
                        if let Some(msg) = &r.message {
                            crate::utils::info(msg);
                        } else {
                            crate::utils::step(&format!("Installed: {}", r.package));
                        }
                    } else {
                        crate::utils::warning(&format!(
                            "Failed to install '{}' via {}: {}",
                            r.package,
                            manager.name(),
                            r.message.as_deref().unwrap_or("unknown error")
                        ));
                    }
                }
            }
            None => {
                crate::utils::warning(&format!(
                    "No package manager available. Skipping {} common package(s).",
                    pkgs.common.len()
                ));
            }
        }
    }

    for manager in &managers {
        let to_install = match manager.field_name() {
            "homebrew" => pkgs.homebrew.clone(),
            "homebrew_casks" => pkgs.homebrew_casks.clone(),
            "apt" => pkgs.apt.clone(),
            "dnf" => pkgs.dnf.clone(),
            "pacman" => pkgs.pacman.clone(),
            "apk" => pkgs.apk.clone(),
            _ => vec![],
        };

        if to_install.is_empty() {
            continue;
        }

        if !manager.is_available() {
            crate::utils::warning(&format!(
                "Package manager '{}' is not available on this system. Skipping {} package(s).",
                manager.name(),
                to_install.len()
            ));
            continue;
        }

        let results = manager.install_many(&to_install, dry_run)?;
        for r in &results {
            if r.success {
                if let Some(msg) = &r.message {
                    crate::utils::info(msg);
                } else {
                    crate::utils::step(&format!("Installed: {}", r.package));
                }
            } else {
                crate::utils::warning(&format!(
                    "Failed to install '{}' via {}: {}",
                    r.package,
                    manager.name(),
                    r.message.as_deref().unwrap_or("unknown error")
                ));
            }
        }
    }

    Ok(())
}
