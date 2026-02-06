use anyhow::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Built-in package name mappings for common tools
/// Format: tool_name -> (apt, brew, dnf, pacman)
static BUILTIN_MAPPINGS: Lazy<HashMap<&'static str, PackageNames>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Core tools (same name everywhere)
    for tool in &["git", "vim", "tmux", "curl", "wget", "tree", "make"] {
        map.insert(*tool, PackageNames::uniform(tool));
    }

    // Tools with same name on most platforms
    map.insert("neovim", PackageNames::uniform("neovim"));
    map.insert("zoxide", PackageNames::uniform("zoxide"));
    map.insert("ripgrep", PackageNames::uniform("ripgrep"));
    map.insert(
        "fd",
        PackageNames::uniform("fd-find")
            .with_brew("fd")
            .with_pacman("fd"),
    );
    map.insert("bat", PackageNames::uniform("bat"));
    map.insert("fzf", PackageNames::uniform("fzf"));
    map.insert("stow", PackageNames::uniform("stow"));
    map.insert("gh", PackageNames::uniform("gh"));

    // Tools with different names
    map.insert(
        "docker",
        PackageNames {
            apt: Some("docker.io".to_string()),
            brew: Some("docker".to_string()),
            dnf: Some("docker".to_string()),
            pacman: Some("docker".to_string()),
        },
    );

    map.insert(
        "gcc",
        PackageNames {
            apt: Some("build-essential".to_string()),
            brew: Some("gcc".to_string()),
            dnf: Some("gcc".to_string()),
            pacman: Some("gcc".to_string()),
        },
    );

    map.insert(
        "kubernetes-cli",
        PackageNames {
            apt: Some("kubectl".to_string()),
            brew: Some("kubectl".to_string()),
            dnf: Some("kubernetes-client".to_string()),
            pacman: Some("kubectl".to_string()),
        },
    );

    map.insert(
        "node",
        PackageNames {
            apt: Some("nodejs".to_string()),
            brew: Some("node".to_string()),
            dnf: Some("nodejs".to_string()),
            pacman: Some("nodejs".to_string()),
        },
    );

    map.insert("python3", PackageNames::uniform("python3"));
    map.insert("rust", PackageNames::uniform("rust"));
    map.insert(
        "go",
        PackageNames {
            apt: Some("golang".to_string()),
            brew: Some("go".to_string()),
            dnf: Some("golang".to_string()),
            pacman: Some("go".to_string()),
        },
    );

    map
});

#[derive(Debug, Clone)]
pub struct PackageNames {
    pub apt: Option<String>,
    pub brew: Option<String>,
    pub dnf: Option<String>,
    pub pacman: Option<String>,
}

impl PackageNames {
    /// Create uniform mapping (same name on all platforms)
    fn uniform(name: &str) -> Self {
        Self {
            apt: Some(name.to_string()),
            brew: Some(name.to_string()),
            dnf: Some(name.to_string()),
            pacman: Some(name.to_string()),
        }
    }

    fn with_brew(mut self, name: &str) -> Self {
        self.brew = Some(name.to_string());
        self
    }

    fn with_pacman(mut self, name: &str) -> Self {
        self.pacman = Some(name.to_string());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerType {
    Apt,
    Homebrew,
    Dnf,
    Pacman,
}

/// Map a tool name to a package name for a specific package manager
pub fn map_package_name(
    tool: &str,
    manager_type: PackageManagerType,
    custom_mappings: &HashMap<String, crate::config::PackageMapping>,
) -> String {
    // First check custom mappings from config
    if let Some(mapping) = custom_mappings.get(tool) {
        let pkg = match manager_type {
            PackageManagerType::Apt => &mapping.apt,
            PackageManagerType::Homebrew => &mapping.brew,
            PackageManagerType::Dnf => &mapping.dnf,
            PackageManagerType::Pacman => &mapping.pacman,
        };
        if let Some(name) = pkg {
            return name.clone();
        }
    }

    // Then check built-in mappings
    if let Some(names) = BUILTIN_MAPPINGS.get(tool) {
        let pkg = match manager_type {
            PackageManagerType::Apt => &names.apt,
            PackageManagerType::Homebrew => &names.brew,
            PackageManagerType::Dnf => &names.dnf,
            PackageManagerType::Pacman => &names.pacman,
        };
        if let Some(name) = pkg {
            return name.clone();
        }
    }

    // Fallback: use tool name as-is
    tool.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_uniform_package() {
        let custom = HashMap::new();
        assert_eq!(
            map_package_name("git", PackageManagerType::Apt, &custom),
            "git"
        );
        assert_eq!(
            map_package_name("git", PackageManagerType::Homebrew, &custom),
            "git"
        );
    }

    #[test]
    fn test_map_docker() {
        let custom = HashMap::new();
        assert_eq!(
            map_package_name("docker", PackageManagerType::Apt, &custom),
            "docker.io"
        );
        assert_eq!(
            map_package_name("docker", PackageManagerType::Homebrew, &custom),
            "docker"
        );
    }

    #[test]
    fn test_custom_mapping() {
        let mut custom = HashMap::new();
        custom.insert(
            "mytool".to_string(),
            crate::config::PackageMapping {
                apt: Some("mytool-apt".to_string()),
                brew: Some("mytool-brew".to_string()),
                dnf: None,
                pacman: None,
            },
        );

        assert_eq!(
            map_package_name("mytool", PackageManagerType::Apt, &custom),
            "mytool-apt"
        );
        assert_eq!(
            map_package_name("mytool", PackageManagerType::Homebrew, &custom),
            "mytool-brew"
        );
    }
}
