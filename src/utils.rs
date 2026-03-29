#![allow(dead_code)]

use colored::Colorize;
use std::borrow::Cow;
use std::path::PathBuf;

// Terminal output
pub fn success(msg: &str) { println!("{} {}", "✓".green().bold(), msg); }
pub fn info(msg: &str)    { println!("{} {}", "ℹ".blue(), msg); }
pub fn warning(msg: &str) { eprintln!("{} {}", "⚠".yellow(), msg); }
pub fn step(msg: &str)    { println!("  {} {}", "→".cyan(), msg); }

#[derive(Debug, PartialEq)]
pub enum LinuxDistro { Debian, Ubuntu, Fedora, Rhel, CentOs, Arch, Manjaro, Alpine, Other }

#[derive(Debug, PartialEq)]
pub enum Os { MacOS, Linux(LinuxDistro), Unknown }

pub fn detect_os() -> Os {
    #[cfg(target_os = "macos")]
    return Os::MacOS;
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let id = content.lines()
                .find(|l| l.starts_with("ID="))
                .map(|l| l.trim_start_matches("ID=").trim_matches('"').to_lowercase());
            return Os::Linux(match id.as_deref() {
                Some("debian")  => LinuxDistro::Debian,
                Some("ubuntu")  => LinuxDistro::Ubuntu,
                Some("fedora")  => LinuxDistro::Fedora,
                Some("rhel")    => LinuxDistro::Rhel,
                Some("centos")  => LinuxDistro::CentOs,
                Some("arch")    => LinuxDistro::Arch,
                Some("manjaro") => LinuxDistro::Manjaro,
                Some("alpine")  => LinuxDistro::Alpine,
                _               => LinuxDistro::Other,
            });
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
    Ok(home_dir()?.join(".dotfiles").join(".heimdal").join("state.json"))
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
