use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatingSystem {
    MacOS,
    Linux(LinuxDistro),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxDistro {
    Debian,
    Ubuntu,
    Fedora,
    RHEL,
    CentOS,
    RedHat,
    Arch,
    Manjaro,
    Other(String),
}

/// Detect the current operating system
pub fn detect_os() -> OperatingSystem {
    if cfg!(target_os = "macos") {
        return OperatingSystem::MacOS;
    }

    if cfg!(target_os = "linux") {
        return OperatingSystem::Linux(detect_linux_distro());
    }

    OperatingSystem::Unknown
}

/// Detect Linux distribution from /etc/os-release
fn detect_linux_distro() -> LinuxDistro {
    // Try to read /etc/os-release
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID=") {
                let id = line
                    .trim_start_matches("ID=")
                    .trim_matches('"')
                    .to_lowercase();
                return match id.as_str() {
                    "debian" => LinuxDistro::Debian,
                    "ubuntu" => LinuxDistro::Ubuntu,
                    "fedora" => LinuxDistro::Fedora,
                    "rhel" => LinuxDistro::RHEL,
                    "centos" => LinuxDistro::CentOS,
                    "redhat" => LinuxDistro::RedHat,
                    "arch" => LinuxDistro::Arch,
                    "manjaro" => LinuxDistro::Manjaro,
                    other => LinuxDistro::Other(other.to_string()),
                };
            }
        }
    }

    // Fallback: try lsb_release
    if let Ok(output) = Command::new("lsb_release").arg("-is").output() {
        if output.status.success() {
            let distro = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            return match distro.as_str() {
                "debian" => LinuxDistro::Debian,
                "ubuntu" => LinuxDistro::Ubuntu,
                "fedora" => LinuxDistro::Fedora,
                "redhat" => LinuxDistro::RedHat,
                "arch" => LinuxDistro::Arch,
                other => LinuxDistro::Other(other.to_string()),
            };
        }
    }

    LinuxDistro::Other("unknown".to_string())
}

/// Check if running on macOS
pub fn is_macos() -> bool {
    matches!(detect_os(), OperatingSystem::MacOS)
}

/// Check if running on Linux
pub fn is_linux() -> bool {
    matches!(detect_os(), OperatingSystem::Linux(_))
}

/// Get OS name as string (for filtering hooks)
pub fn os_name() -> String {
    match detect_os() {
        OperatingSystem::MacOS => "macos".to_string(),
        OperatingSystem::Linux(_) => "linux".to_string(),
        OperatingSystem::Unknown => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let os = detect_os();
        assert!(!matches!(os, OperatingSystem::Unknown));
    }

    #[test]
    fn test_os_name() {
        let name = os_name();
        assert!(name == "macos" || name == "linux");
    }
}
