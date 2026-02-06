use once_cell::sync::Lazy;
use std::collections::HashMap;
use strsim::jaro_winkler;

/// Built-in package name mappings for common tools
/// Format: tool_name -> (apt, brew, dnf, pacman)
static BUILTIN_MAPPINGS: Lazy<HashMap<&'static str, PackageNames>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Core tools (same name everywhere)
    for tool in &[
        "git", "vim", "tmux", "curl", "wget", "tree", "make", "jq", "stow",
    ] {
        map.insert(*tool, PackageNames::uniform(tool));
    }

    // Editors
    map.insert("neovim", PackageNames::uniform("neovim"));
    map.insert("emacs", PackageNames::uniform("emacs"));

    // Terminal utilities - same name everywhere
    map.insert("zoxide", PackageNames::uniform("zoxide"));
    map.insert("ripgrep", PackageNames::uniform("ripgrep"));
    map.insert("bat", PackageNames::uniform("bat"));
    map.insert("fzf", PackageNames::uniform("fzf"));
    map.insert("htop", PackageNames::uniform("htop"));
    map.insert("zsh", PackageNames::uniform("zsh"));
    map.insert("starship", PackageNames::uniform("starship"));
    map.insert("screen", PackageNames::uniform("screen"));
    map.insert("zellij", PackageNames::uniform("zellij"));
    map.insert("skim", PackageNames::uniform("skim"));
    map.insert("glances", PackageNames::uniform("glances"));

    // fd - different names on some platforms
    map.insert(
        "fd",
        PackageNames::uniform("fd-find")
            .with_brew("fd")
            .with_pacman("fd"),
    );

    // Git tools
    map.insert("gh", PackageNames::uniform("gh"));
    map.insert("delta", PackageNames::uniform("git-delta"));
    map.insert("lazygit", PackageNames::uniform("lazygit"));
    map.insert("tig", PackageNames::uniform("tig"));
    map.insert("hub", PackageNames::uniform("hub"));

    // Docker - docker.io on apt
    map.insert(
        "docker",
        PackageNames {
            apt: Some("docker.io".to_string()),
            brew: Some("docker".to_string()),
            dnf: Some("docker".to_string()),
            pacman: Some("docker".to_string()),
        },
    );

    map.insert("docker-compose", PackageNames::uniform("docker-compose"));
    map.insert("podman", PackageNames::uniform("podman"));

    // Kubernetes tools
    map.insert(
        "kubectl",
        PackageNames {
            apt: Some("kubectl".to_string()),
            brew: Some("kubectl".to_string()),
            dnf: Some("kubernetes-client".to_string()),
            pacman: Some("kubectl".to_string()),
        },
    );

    map.insert("helm", PackageNames::uniform("helm"));
    map.insert("k9s", PackageNames::uniform("k9s"));

    // Build tools
    map.insert(
        "gcc",
        PackageNames {
            apt: Some("build-essential".to_string()),
            brew: Some("gcc".to_string()),
            dnf: Some("gcc".to_string()),
            pacman: Some("gcc".to_string()),
        },
    );

    map.insert("cmake", PackageNames::uniform("cmake"));
    map.insert(
        "ninja",
        PackageNames::uniform("ninja-build").with_brew("ninja"),
    );

    // Programming languages
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
    map.insert(
        "python",
        PackageNames {
            apt: Some("python3".to_string()),
            brew: Some("python".to_string()),
            dnf: Some("python3".to_string()),
            pacman: Some("python".to_string()),
        },
    );

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

    // Language package managers
    map.insert("npm", PackageNames::uniform("npm"));
    map.insert("yarn", PackageNames::uniform("yarn"));
    map.insert("pnpm", PackageNames::uniform("pnpm"));
    map.insert("pip", PackageNames::uniform("python3-pip").with_brew("pip"));
    map.insert("pipenv", PackageNames::uniform("pipenv"));
    map.insert("poetry", PackageNames::uniform("poetry"));
    map.insert("cargo", PackageNames::uniform("cargo"));

    // Infrastructure tools
    map.insert("terraform", PackageNames::uniform("terraform"));
    map.insert("ansible", PackageNames::uniform("ansible"));
    map.insert("pulumi", PackageNames::uniform("pulumi"));

    // Databases
    map.insert(
        "postgresql",
        PackageNames {
            apt: Some("postgresql".to_string()),
            brew: Some("postgresql".to_string()),
            dnf: Some("postgresql-server".to_string()),
            pacman: Some("postgresql".to_string()),
        },
    );

    map.insert("redis", PackageNames::uniform("redis"));
    map.insert(
        "mysql",
        PackageNames {
            apt: Some("mysql-server".to_string()),
            brew: Some("mysql".to_string()),
            dnf: Some("mysql-server".to_string()),
            pacman: Some("mysql".to_string()),
        },
    );
    map.insert(
        "sqlite",
        PackageNames::uniform("sqlite3").with_brew("sqlite"),
    );
    map.insert("memcached", PackageNames::uniform("memcached"));

    // Network tools
    map.insert("httpie", PackageNames::uniform("httpie"));
    map.insert("netcat", PackageNames::uniform("netcat"));
    map.insert(
        "nmap",
        PackageNames {
            apt: Some("nmap".to_string()),
            brew: Some("nmap".to_string()),
            dnf: Some("nmap".to_string()),
            pacman: Some("nmap".to_string()),
        },
    );

    // Documentation
    map.insert("pandoc", PackageNames::uniform("pandoc"));

    // Modern alternatives to classic tools
    map.insert("exa", PackageNames::uniform("exa"));
    map.insert("btop", PackageNames::uniform("btop"));
    map.insert(
        "ag",
        PackageNames::uniform("silversearcher-ag").with_brew("the_silver_searcher"),
    );

    // JavaScript runtimes
    map.insert("deno", PackageNames::uniform("deno"));
    map.insert("bun", PackageNames::uniform("bun"));

    // Language-specific tools
    map.insert("pyenv", PackageNames::uniform("pyenv"));
    map.insert("gopls", PackageNames::uniform("gopls"));
    map.insert("rust-analyzer", PackageNames::uniform("rust-analyzer"));

    // Other utilities
    map.insert("tmuxinator", PackageNames::uniform("tmuxinator"));
    map.insert("yq", PackageNames::uniform("yq"));
    map.insert("watch", PackageNames::uniform("watch"));
    map.insert("pgcli", PackageNames::uniform("pgcli"));
    map.insert("tflint", PackageNames::uniform("tflint"));

    map
});

/// Name normalization mappings (common aliases -> canonical names)
static NAME_NORMALIZATIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Node.js variations
    map.insert("nodejs", "node");
    map.insert("node.js", "node");
    map.insert("node-js", "node");

    // Go variations
    map.insert("golang", "go");
    map.insert("go-lang", "go");

    // Python variations
    // Note: 'python3' is kept separate in BUILTIN_MAPPINGS to allow platform-specific handling
    // Only map common shorthand aliases here
    map.insert("py", "python");
    map.insert("py3", "python");

    // Docker variations
    map.insert("docker.io", "docker");
    map.insert("docker-ce", "docker");

    // Kubernetes variations
    map.insert("k8s", "kubectl");
    map.insert("kubernetes", "kubectl");
    map.insert("kubernetes-cli", "kubectl");

    // PostgreSQL variations
    map.insert("postgres", "postgresql");
    map.insert("pg", "postgresql");
    map.insert("psql", "postgresql");

    // Grep variations
    map.insert("rg", "ripgrep");

    // Neovim alias
    map.insert("nvim", "neovim");

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

    #[allow(dead_code)]
    fn with_apt(mut self, name: &str) -> Self {
        self.apt = Some(name.to_string());
        self
    }

    #[allow(dead_code)]
    fn with_dnf(mut self, name: &str) -> Self {
        self.dnf = Some(name.to_string());
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

/// Normalize a package name to its canonical form
///
/// Examples:
/// - "nodejs" -> "node"
/// - "golang" -> "go"
/// - "rg" -> "ripgrep"
pub fn normalize_package_name(name: &str) -> String {
    let lower = name.to_lowercase();
    NAME_NORMALIZATIONS
        .get(lower.as_str())
        .map(|&s| s.to_string())
        .unwrap_or_else(|| name.to_string())
}

/// Find the best fuzzy match for a package name
///
/// Returns the best matching package name if the similarity is above 0.85
/// Uses Jaro-Winkler distance for similarity calculation
pub fn fuzzy_match_package(query: &str, available: &[&str]) -> Option<String> {
    let query_lower = query.to_lowercase();
    let mut best_match = None;
    let mut best_score = 0.0;

    for &pkg in available {
        let pkg_lower = pkg.to_lowercase();
        let score = jaro_winkler(&query_lower, &pkg_lower);

        if score > best_score && score > 0.85 {
            best_score = score;
            best_match = Some(pkg.to_string());
        }
    }

    best_match
}

/// Get all available package names for fuzzy matching
pub fn get_available_packages() -> Vec<&'static str> {
    BUILTIN_MAPPINGS.keys().copied().collect()
}

/// Find suggestions for a misspelled or unknown package
///
/// First tries normalization, then fuzzy matching
pub fn suggest_package(name: &str) -> Option<String> {
    // Try normalization first
    let normalized = normalize_package_name(name);
    if BUILTIN_MAPPINGS.contains_key(normalized.as_str()) && normalized != name {
        return Some(normalized);
    }

    // Try fuzzy matching
    let available: Vec<&str> = get_available_packages();
    fuzzy_match_package(name, &available)
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

    // Try to normalize the package name first
    let normalized = normalize_package_name(tool);

    // Then check built-in mappings with normalized name
    if let Some(names) = BUILTIN_MAPPINGS.get(normalized.as_str()) {
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

    // Try with original name if normalization didn't work
    if normalized != tool {
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
    fn test_map_node() {
        let custom = HashMap::new();
        assert_eq!(
            map_package_name("node", PackageManagerType::Apt, &custom),
            "nodejs"
        );
        assert_eq!(
            map_package_name("node", PackageManagerType::Homebrew, &custom),
            "node"
        );
    }

    #[test]
    fn test_map_kubectl() {
        let custom = HashMap::new();
        assert_eq!(
            map_package_name("kubectl", PackageManagerType::Dnf, &custom),
            "kubernetes-client"
        );
        assert_eq!(
            map_package_name("kubectl", PackageManagerType::Apt, &custom),
            "kubectl"
        );
    }

    #[test]
    fn test_map_fd() {
        let custom = HashMap::new();
        assert_eq!(
            map_package_name("fd", PackageManagerType::Apt, &custom),
            "fd-find"
        );
        assert_eq!(
            map_package_name("fd", PackageManagerType::Homebrew, &custom),
            "fd"
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

    #[test]
    fn test_normalize_nodejs() {
        assert_eq!(normalize_package_name("nodejs"), "node");
        assert_eq!(normalize_package_name("node.js"), "node");
        assert_eq!(normalize_package_name("node-js"), "node");
        assert_eq!(normalize_package_name("node"), "node");
    }

    #[test]
    fn test_normalize_golang() {
        assert_eq!(normalize_package_name("golang"), "go");
        assert_eq!(normalize_package_name("go-lang"), "go");
        assert_eq!(normalize_package_name("go"), "go");
    }

    #[test]
    fn test_normalize_postgres() {
        assert_eq!(normalize_package_name("postgres"), "postgresql");
        assert_eq!(normalize_package_name("pg"), "postgresql");
        assert_eq!(normalize_package_name("psql"), "postgresql");
    }

    #[test]
    fn test_normalize_k8s() {
        assert_eq!(normalize_package_name("k8s"), "kubectl");
        assert_eq!(normalize_package_name("kubernetes"), "kubectl");
    }

    #[test]
    fn test_normalize_ripgrep() {
        assert_eq!(normalize_package_name("rg"), "ripgrep");
    }

    #[test]
    fn test_normalize_neovim() {
        assert_eq!(normalize_package_name("nvim"), "neovim");
    }

    #[test]
    fn test_fuzzy_match_exact() {
        let available = vec!["ripgrep", "git", "docker"];
        assert_eq!(
            fuzzy_match_package("ripgrep", &available),
            Some("ripgrep".to_string())
        );
    }

    #[test]
    fn test_fuzzy_match_typo() {
        let available = vec!["ripgrep", "git", "docker"];
        assert_eq!(
            fuzzy_match_package("ripgrap", &available),
            Some("ripgrep".to_string())
        );
        assert_eq!(
            fuzzy_match_package("ripgre", &available),
            Some("ripgrep".to_string())
        );
    }

    #[test]
    fn test_fuzzy_match_docker_typos() {
        let available = vec!["docker", "git", "kubectl"];
        assert_eq!(
            fuzzy_match_package("dokcer", &available),
            Some("docker".to_string())
        );
        assert_eq!(
            fuzzy_match_package("doker", &available),
            Some("docker".to_string())
        );
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let available = vec!["ripgrep", "git", "docker"];
        assert_eq!(fuzzy_match_package("xyz", &available), None);
        assert_eq!(
            fuzzy_match_package("completely-different", &available),
            None
        );
    }

    #[test]
    fn test_fuzzy_match_kubectl() {
        let available = vec!["kubectl", "helm", "k9s"];
        assert_eq!(
            fuzzy_match_package("kubctl", &available),
            Some("kubectl".to_string())
        );
        assert_eq!(
            fuzzy_match_package("kubeclt", &available),
            Some("kubectl".to_string())
        );
    }

    #[test]
    fn test_suggest_package_normalization() {
        // Should suggest normalized name
        assert_eq!(suggest_package("nodejs"), Some("node".to_string()));
        assert_eq!(suggest_package("golang"), Some("go".to_string()));
    }

    #[test]
    fn test_suggest_package_fuzzy() {
        // Should suggest via fuzzy matching
        assert_eq!(suggest_package("ripgrap"), Some("ripgrep".to_string()));
        assert_eq!(suggest_package("dokcer"), Some("docker".to_string()));
    }

    #[test]
    fn test_suggest_package_no_suggestion() {
        // Should return None for completely different names
        assert_eq!(suggest_package("completely-unknown-package"), None);
    }

    #[test]
    fn test_map_with_normalization() {
        let custom = HashMap::new();
        // "nodejs" should normalize to "node" and map to "nodejs" on apt
        assert_eq!(
            map_package_name("nodejs", PackageManagerType::Apt, &custom),
            "nodejs"
        );
        assert_eq!(
            map_package_name("golang", PackageManagerType::Apt, &custom),
            "golang"
        );
    }

    #[test]
    fn test_all_database_packages_mapped() {
        // Verify all common packages from database have mappings
        let custom = HashMap::new();
        let packages = vec![
            "git",
            "curl",
            "wget",
            "neovim",
            "vim",
            "tmux",
            "ripgrep",
            "bat",
            "fd",
            "fzf",
            "jq",
            "htop",
            "tree",
            "node",
            "python",
            "go",
            "rust",
            "docker",
            "docker-compose",
            "kubectl",
            "helm",
            "k9s",
            "terraform",
            "ansible",
            "gh",
            "delta",
            "lazygit",
            "make",
            "cmake",
            "postgresql",
            "redis",
            "yarn",
            "pipenv",
            "zsh",
            "starship",
            "pandoc",
        ];

        for pkg in packages {
            let result = map_package_name(pkg, PackageManagerType::Apt, &custom);
            // Should return something (not panic or return empty)
            assert!(!result.is_empty(), "Package {} should have a mapping", pkg);
        }
    }

    #[test]
    fn test_get_available_packages() {
        let packages = get_available_packages();
        assert!(packages.len() > 40, "Should have 40+ packages");
        assert!(packages.contains(&"git"));
        assert!(packages.contains(&"docker"));
        assert!(packages.contains(&"node"));
    }
}
