#![allow(dead_code)]

use colored::Colorize;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// Atomically write content to a file using temp file + rename pattern.
/// Prevents partial writes and corruption.
pub fn atomic_write(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Ensure parent directory exists before writing a file.
pub fn ensure_parent_exists(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Get the system hostname as a String.
pub fn hostname() -> String {
    hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

// Terminal output
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}
pub fn warning(msg: &str) {
    eprintln!("{} {}", "⚠".yellow(), msg);
}
pub fn step(msg: &str) {
    println!("  {} {}", "→".cyan(), msg);
}

#[derive(Debug, PartialEq)]
pub enum LinuxDistro {
    Debian,
    Ubuntu,
    Fedora,
    Rhel,
    CentOs,
    Arch,
    Manjaro,
    Alpine,
    Other,
}

#[derive(Debug, PartialEq)]
pub enum Os {
    MacOS,
    Linux(LinuxDistro),
    Unknown,
}

pub fn detect_os() -> Os {
    #[cfg(target_os = "macos")]
    return Os::MacOS;
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            // First try ID=
            let id = content
                .lines()
                .find(|l| l.starts_with("ID="))
                .map(|l| l.trim_start_matches("ID=").trim_matches('"').to_lowercase());

            let distro = match id.as_deref() {
                Some("debian") => Some(LinuxDistro::Debian),
                Some("ubuntu") => Some(LinuxDistro::Ubuntu),
                Some("fedora") => Some(LinuxDistro::Fedora),
                Some("rhel") => Some(LinuxDistro::Rhel),
                Some("centos") => Some(LinuxDistro::CentOs),
                Some("arch") => Some(LinuxDistro::Arch),
                Some("manjaro") => Some(LinuxDistro::Manjaro),
                Some("alpine") => Some(LinuxDistro::Alpine),
                _ => None,
            };

            if let Some(d) = distro {
                return Os::Linux(d);
            }

            // Fallback: check ID_LIKE= for derived distros (e.g. Linux Mint, Pop!_OS)
            let id_like = content
                .lines()
                .find(|l| l.starts_with("ID_LIKE="))
                .map(|l| {
                    l.trim_start_matches("ID_LIKE=")
                        .trim_matches('"')
                        .to_lowercase()
                });

            if let Some(like_str) = id_like {
                for part in like_str.split_whitespace() {
                    let d = match part {
                        "debian" => Some(LinuxDistro::Debian),
                        "ubuntu" => Some(LinuxDistro::Ubuntu),
                        "fedora" => Some(LinuxDistro::Fedora),
                        "rhel" => Some(LinuxDistro::Rhel),
                        "centos" => Some(LinuxDistro::CentOs),
                        "arch" => Some(LinuxDistro::Arch),
                        _ => None,
                    };
                    if let Some(d) = d {
                        return Os::Linux(d);
                    }
                }
            }

            return Os::Linux(LinuxDistro::Other);
        }
        return Os::Linux(LinuxDistro::Other);
    }
    #[allow(unreachable_code)]
    Os::Unknown
}

pub fn os_name() -> &'static str {
    match detect_os() {
        Os::MacOS => "macos",
        Os::Linux(_) => "linux",
        Os::Unknown => "unknown",
    }
}

pub fn expand_path(p: &str) -> PathBuf {
    PathBuf::from(shellexpand::full(p).unwrap_or(Cow::Borrowed(p)).as_ref())
}

pub fn home_dir() -> anyhow::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))
}

pub fn dotfiles_dir() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(".dotfiles"))
}

pub fn state_path() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(".heimdal").join("state.json"))
}

pub fn confirm(prompt: &str) -> bool {
    dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .unwrap_or(false)
}

pub fn prompt_string(prompt: &str, default: &str) -> String {
    dialoguer::Input::new()
        .with_prompt(prompt)
        .default(default.to_string())
        .interact_text()
        .unwrap_or_else(|_| default.to_string())
}
