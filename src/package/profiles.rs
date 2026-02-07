use crate::utils::{detect_os, LinuxDistro, OperatingSystem};

/// Pre-defined package profiles for common use cases
#[derive(Debug, Clone, PartialEq)]
pub enum ProfileType {
    /// Minimal set of essential tools
    Minimal,
    /// General developer tools
    Developer,
    /// Web development (Node, npm, frontend)
    WebDev,
    /// Rust development tools
    RustDev,
    /// Python development tools
    PythonDev,
    /// Go development tools
    GoDev,
    /// DevOps and infrastructure tools
    DevOps,
    /// Data science and ML tools
    DataScience,
    /// Designer tools (macOS only)
    Designer,
    /// Writer/content creator tools
    Writer,
    /// Custom profile with user-defined packages
    #[allow(dead_code)]
    Custom(String),
}

impl ProfileType {
    /// Get all available profile types (excluding Custom)
    pub fn all() -> Vec<ProfileType> {
        vec![
            ProfileType::Minimal,
            ProfileType::Developer,
            ProfileType::WebDev,
            ProfileType::RustDev,
            ProfileType::PythonDev,
            ProfileType::GoDev,
            ProfileType::DevOps,
            ProfileType::DataScience,
            ProfileType::Designer,
            ProfileType::Writer,
        ]
    }

    /// Get the display name for the profile
    pub fn display_name(&self) -> &str {
        match self {
            ProfileType::Minimal => "Minimal",
            ProfileType::Developer => "Developer",
            ProfileType::WebDev => "Web Developer",
            ProfileType::RustDev => "Rust Developer",
            ProfileType::PythonDev => "Python Developer",
            ProfileType::GoDev => "Go Developer",
            ProfileType::DevOps => "DevOps Engineer",
            ProfileType::DataScience => "Data Scientist",
            ProfileType::Designer => "Designer",
            ProfileType::Writer => "Writer",
            ProfileType::Custom(name) => name,
        }
    }

    /// Get a description of what this profile includes
    pub fn description(&self) -> &str {
        match self {
            ProfileType::Minimal => "Essential command-line tools (git, vim, tmux, curl)",
            ProfileType::Developer => {
                "General software development tools (editors, build tools, version control)"
            }
            ProfileType::WebDev => "Web development stack (Node.js, npm, frontend build tools)",
            ProfileType::RustDev => "Rust toolchain and cargo utilities",
            ProfileType::PythonDev => "Python ecosystem and common libraries",
            ProfileType::GoDev => "Go toolchain and development tools",
            ProfileType::DevOps => {
                "Infrastructure tools (Docker, Kubernetes, Terraform, cloud CLIs)"
            }
            ProfileType::DataScience => {
                "Data analysis and machine learning tools (Python, R, Jupyter)"
            }
            ProfileType::Designer => "Design and creative tools (macOS apps)",
            ProfileType::Writer => "Writing and documentation tools (pandoc, markdown, LaTeX)",
            ProfileType::Custom(_) => "Custom package selection",
        }
    }

    /// Check if this profile is available on the current platform
    pub fn is_available(&self) -> bool {
        let os = detect_os();
        match self {
            // Designer profile is macOS-only
            ProfileType::Designer => matches!(os, OperatingSystem::MacOS),
            // All other profiles work on any platform
            _ => true,
        }
    }
}

/// A package profile with platform-specific package lists
#[derive(Debug, Clone)]
pub struct PackageProfile {
    #[allow(dead_code)]
    pub profile_type: ProfileType,
    pub base_packages: Vec<String>,
    pub macos_packages: Vec<String>,
    pub linux_packages: Vec<String>,
    pub debian_packages: Vec<String>,
    pub arch_packages: Vec<String>,
    pub fedora_packages: Vec<String>,
}

impl PackageProfile {
    /// Create a profile from a profile type
    pub fn from_type(profile_type: ProfileType) -> Self {
        match &profile_type {
            ProfileType::Minimal => Self::minimal(),
            ProfileType::Developer => Self::developer(),
            ProfileType::WebDev => Self::web_dev(),
            ProfileType::RustDev => Self::rust_dev(),
            ProfileType::PythonDev => Self::python_dev(),
            ProfileType::GoDev => Self::go_dev(),
            ProfileType::DevOps => Self::devops(),
            ProfileType::DataScience => Self::data_science(),
            ProfileType::Designer => Self::designer(),
            ProfileType::Writer => Self::writer(),
            ProfileType::Custom(_) => Self::empty(profile_type),
        }
    }

    /// Get all packages for the current platform
    pub fn resolve_packages(&self) -> Vec<String> {
        let mut packages = self.base_packages.clone();
        let os = detect_os();

        match os {
            OperatingSystem::MacOS => {
                packages.extend(self.macos_packages.clone());
            }
            OperatingSystem::Linux(distro) => {
                packages.extend(self.linux_packages.clone());

                match distro {
                    LinuxDistro::Debian | LinuxDistro::Ubuntu => {
                        packages.extend(self.debian_packages.clone());
                    }
                    LinuxDistro::Arch | LinuxDistro::Manjaro => {
                        packages.extend(self.arch_packages.clone());
                    }
                    LinuxDistro::Fedora | LinuxDistro::Rhel | LinuxDistro::CentOS => {
                        packages.extend(self.fedora_packages.clone());
                    }
                    _ => {}
                }
            }
            OperatingSystem::Unknown => {}
        }

        // Remove duplicates and sort
        packages.sort();
        packages.dedup();
        packages
    }

    /// Get package count for the current platform
    #[allow(dead_code)]
    pub fn package_count(&self) -> usize {
        self.resolve_packages().len()
    }

    // Profile definitions

    fn minimal() -> Self {
        Self {
            profile_type: ProfileType::Minimal,
            base_packages: vec![
                "git".to_string(),
                "vim".to_string(),
                "curl".to_string(),
                "wget".to_string(),
            ],
            macos_packages: vec!["tmux".to_string()],
            linux_packages: vec!["tmux".to_string()],
            debian_packages: vec![],
            arch_packages: vec![],
            fedora_packages: vec![],
        }
    }

    fn developer() -> Self {
        Self {
            profile_type: ProfileType::Developer,
            base_packages: vec![
                "git".to_string(),
                "neovim".to_string(),
                "tmux".to_string(),
                "fzf".to_string(),
                "ripgrep".to_string(),
                "bat".to_string(),
                "fd".to_string(),
                "jq".to_string(),
                "htop".to_string(),
                "tree".to_string(),
            ],
            macos_packages: vec![
                "gh".to_string(), // GitHub CLI
            ],
            linux_packages: vec!["gh".to_string()],
            debian_packages: vec!["build-essential".to_string()],
            arch_packages: vec!["base-devel".to_string()],
            fedora_packages: vec!["gcc".to_string(), "make".to_string()],
        }
    }

    fn web_dev() -> Self {
        Self {
            profile_type: ProfileType::WebDev,
            base_packages: vec![
                "git".to_string(),
                "neovim".to_string(),
                "node".to_string(),
                "tmux".to_string(),
                "fzf".to_string(),
                "ripgrep".to_string(),
            ],
            macos_packages: vec!["gh".to_string(), "yarn".to_string()],
            linux_packages: vec!["gh".to_string(), "yarn".to_string()],
            debian_packages: vec!["build-essential".to_string()],
            arch_packages: vec!["base-devel".to_string()],
            fedora_packages: vec!["gcc".to_string(), "make".to_string()],
        }
    }

    fn rust_dev() -> Self {
        Self {
            profile_type: ProfileType::RustDev,
            base_packages: vec![
                "git".to_string(),
                "neovim".to_string(),
                "tmux".to_string(),
                "fzf".to_string(),
                "ripgrep".to_string(),
                "bat".to_string(),
                "fd".to_string(),
            ],
            macos_packages: vec!["gh".to_string(), "rust-analyzer".to_string()],
            linux_packages: vec!["gh".to_string()],
            debian_packages: vec![
                "build-essential".to_string(),
                "pkg-config".to_string(),
                "libssl-dev".to_string(),
            ],
            arch_packages: vec!["base-devel".to_string(), "rust-analyzer".to_string()],
            fedora_packages: vec![
                "gcc".to_string(),
                "make".to_string(),
                "openssl-devel".to_string(),
            ],
        }
    }

    fn python_dev() -> Self {
        Self {
            profile_type: ProfileType::PythonDev,
            base_packages: vec![
                "git".to_string(),
                "neovim".to_string(),
                "python".to_string(),
                "tmux".to_string(),
                "fzf".to_string(),
                "ripgrep".to_string(),
            ],
            macos_packages: vec!["gh".to_string(), "pipenv".to_string(), "pyenv".to_string()],
            linux_packages: vec!["gh".to_string()],
            debian_packages: vec![
                "python3-pip".to_string(),
                "python3-venv".to_string(),
                "build-essential".to_string(),
            ],
            arch_packages: vec!["python-pip".to_string(), "base-devel".to_string()],
            fedora_packages: vec![
                "python3-pip".to_string(),
                "gcc".to_string(),
                "make".to_string(),
            ],
        }
    }

    fn go_dev() -> Self {
        Self {
            profile_type: ProfileType::GoDev,
            base_packages: vec![
                "git".to_string(),
                "neovim".to_string(),
                "go".to_string(),
                "tmux".to_string(),
                "fzf".to_string(),
                "ripgrep".to_string(),
            ],
            macos_packages: vec!["gh".to_string()],
            linux_packages: vec!["gh".to_string()],
            debian_packages: vec!["build-essential".to_string()],
            arch_packages: vec!["base-devel".to_string()],
            fedora_packages: vec!["gcc".to_string(), "make".to_string()],
        }
    }

    fn devops() -> Self {
        Self {
            profile_type: ProfileType::DevOps,
            base_packages: vec![
                "git".to_string(),
                "neovim".to_string(),
                "tmux".to_string(),
                "docker".to_string(),
                "kubectl".to_string(),
                "terraform".to_string(),
                "jq".to_string(),
                "curl".to_string(),
                "htop".to_string(),
            ],
            macos_packages: vec!["gh".to_string(), "helm".to_string(), "k9s".to_string()],
            linux_packages: vec!["gh".to_string(), "helm".to_string(), "k9s".to_string()],
            debian_packages: vec![],
            arch_packages: vec![],
            fedora_packages: vec![],
        }
    }

    fn data_science() -> Self {
        Self {
            profile_type: ProfileType::DataScience,
            base_packages: vec![
                "git".to_string(),
                "python".to_string(),
                "neovim".to_string(),
                "tmux".to_string(),
                "jq".to_string(),
            ],
            macos_packages: vec!["gh".to_string(), "jupyterlab".to_string(), "r".to_string()],
            linux_packages: vec!["gh".to_string()],
            debian_packages: vec!["python3-pip".to_string(), "python3-venv".to_string()],
            arch_packages: vec!["python-pip".to_string(), "jupyter-notebook".to_string()],
            fedora_packages: vec!["python3-pip".to_string()],
        }
    }

    fn designer() -> Self {
        Self {
            profile_type: ProfileType::Designer,
            base_packages: vec!["git".to_string()],
            macos_packages: vec![
                "figma".to_string(),
                "imagemagick".to_string(),
                "ffmpeg".to_string(),
            ],
            linux_packages: vec![],
            debian_packages: vec![],
            arch_packages: vec![],
            fedora_packages: vec![],
        }
    }

    fn writer() -> Self {
        Self {
            profile_type: ProfileType::Writer,
            base_packages: vec![
                "git".to_string(),
                "neovim".to_string(),
                "pandoc".to_string(),
            ],
            macos_packages: vec!["gh".to_string(), "marked".to_string()],
            linux_packages: vec!["gh".to_string()],
            debian_packages: vec!["texlive".to_string()],
            arch_packages: vec!["texlive-core".to_string()],
            fedora_packages: vec!["texlive".to_string()],
        }
    }

    fn empty(profile_type: ProfileType) -> Self {
        Self {
            profile_type,
            base_packages: vec![],
            macos_packages: vec![],
            linux_packages: vec![],
            debian_packages: vec![],
            arch_packages: vec![],
            fedora_packages: vec![],
        }
    }
}

/// Profile selector helper for interactive wizards
pub struct ProfileSelector {
    profiles: Vec<ProfileType>,
}

impl ProfileSelector {
    /// Create a new profile selector with all available profiles
    pub fn new() -> Self {
        Self {
            profiles: ProfileType::all()
                .into_iter()
                .filter(|p| p.is_available())
                .collect(),
        }
    }

    /// Get available profile options for display
    pub fn options(&self) -> Vec<(String, String)> {
        self.profiles
            .iter()
            .map(|p| (p.display_name().to_string(), p.description().to_string()))
            .collect()
    }

    /// Get a profile by its display name
    pub fn get_by_name(&self, name: &str) -> Option<ProfileType> {
        self.profiles
            .iter()
            .find(|p| p.display_name() == name)
            .cloned()
    }

    /// Get all profile types
    #[allow(dead_code)]
    pub fn get_all(&self) -> &[ProfileType] {
        &self.profiles
    }
}

impl Default for ProfileSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_display_names() {
        assert_eq!(ProfileType::Minimal.display_name(), "Minimal");
        assert_eq!(ProfileType::Developer.display_name(), "Developer");
        assert_eq!(ProfileType::WebDev.display_name(), "Web Developer");
    }

    #[test]
    fn test_profile_descriptions() {
        let desc = ProfileType::Minimal.description();
        assert!(desc.contains("Essential"));

        let desc = ProfileType::Developer.description();
        assert!(desc.contains("development"));
    }

    #[test]
    fn test_minimal_profile() {
        let profile = PackageProfile::minimal();
        assert_eq!(profile.profile_type, ProfileType::Minimal);
        assert!(profile.base_packages.contains(&"git".to_string()));
        assert!(profile.base_packages.contains(&"vim".to_string()));
    }

    #[test]
    fn test_developer_profile() {
        let profile = PackageProfile::developer();
        assert_eq!(profile.profile_type, ProfileType::Developer);
        assert!(profile.base_packages.contains(&"neovim".to_string()));
        assert!(profile.base_packages.contains(&"ripgrep".to_string()));
    }

    #[test]
    fn test_resolve_packages() {
        let profile = PackageProfile::minimal();
        let packages = profile.resolve_packages();

        // Should include base packages
        assert!(packages.contains(&"git".to_string()));
        assert!(packages.contains(&"vim".to_string()));

        // Should include platform-specific packages
        // (actual packages depend on test environment)
        assert!(!packages.is_empty());
    }

    #[test]
    fn test_package_count() {
        let profile = PackageProfile::minimal();
        assert!(profile.package_count() >= 4); // At least the base packages
    }

    #[test]
    fn test_profile_from_type() {
        let profile = PackageProfile::from_type(ProfileType::Minimal);
        assert_eq!(profile.profile_type, ProfileType::Minimal);

        let profile = PackageProfile::from_type(ProfileType::Developer);
        assert_eq!(profile.profile_type, ProfileType::Developer);
    }

    #[test]
    fn test_profile_selector() {
        let selector = ProfileSelector::new();
        let options = selector.options();

        // Should have multiple profiles
        assert!(options.len() >= 5);

        // Each option should have a name and description
        for (name, desc) in options {
            assert!(!name.is_empty());
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_profile_selector_get_by_name() {
        let selector = ProfileSelector::new();

        let profile = selector.get_by_name("Minimal");
        assert!(profile.is_some());
        assert_eq!(profile.unwrap(), ProfileType::Minimal);

        let profile = selector.get_by_name("Developer");
        assert!(profile.is_some());
        assert_eq!(profile.unwrap(), ProfileType::Developer);

        let profile = selector.get_by_name("NonExistent");
        assert!(profile.is_none());
    }

    #[test]
    fn test_all_profiles_have_git() {
        // All profiles should include git (except empty custom)
        for profile_type in ProfileType::all() {
            if profile_type != ProfileType::Custom("test".to_string()) {
                let profile = PackageProfile::from_type(profile_type);
                let packages = profile.resolve_packages();
                assert!(
                    packages.contains(&"git".to_string()),
                    "Profile {:?} should include git",
                    profile.profile_type
                );
            }
        }
    }

    #[test]
    fn test_web_dev_has_node() {
        let profile = PackageProfile::web_dev();
        let packages = profile.resolve_packages();
        assert!(packages.contains(&"node".to_string()));
    }

    #[test]
    fn test_rust_dev_has_rust_tools() {
        let profile = PackageProfile::rust_dev();
        let packages = profile.resolve_packages();
        assert!(packages.contains(&"ripgrep".to_string()));
        assert!(packages.contains(&"bat".to_string()));
        assert!(packages.contains(&"fd".to_string()));
    }

    #[test]
    fn test_python_dev_has_python() {
        let profile = PackageProfile::python_dev();
        let packages = profile.resolve_packages();
        assert!(packages.contains(&"python".to_string()));
    }

    #[test]
    fn test_devops_has_docker() {
        let profile = PackageProfile::devops();
        let packages = profile.resolve_packages();
        assert!(packages.contains(&"docker".to_string()));
        assert!(packages.contains(&"kubectl".to_string()));
    }
}
