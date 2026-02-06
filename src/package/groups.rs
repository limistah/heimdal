//! Package groups - curated collections of related packages
//!
//! This module provides predefined groups of packages for common development
//! scenarios, making it easy to install everything needed for a specific
//! workflow in one command.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A collection of related packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageGroup {
    /// Group identifier (e.g., "web-dev", "rust-dev")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what this group provides
    pub description: String,

    /// Category (development, productivity, system, etc.)
    pub category: String,

    /// List of package names in this group
    pub packages: Vec<String>,

    /// Optional packages (suggested but not required)
    pub optional_packages: Vec<String>,
}

impl PackageGroup {
    /// Creates a new package group
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        packages: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category: category.into(),
            packages,
            optional_packages: Vec::new(),
        }
    }

    /// Add optional packages to the group
    pub fn with_optional(mut self, optional: Vec<String>) -> Self {
        self.optional_packages = optional;
        self
    }

    /// Get all packages (required + optional)
    pub fn all_packages(&self) -> Vec<&str> {
        self.packages
            .iter()
            .chain(self.optional_packages.iter())
            .map(|s| s.as_str())
            .collect()
    }
}

/// Registry of all available package groups
pub struct GroupRegistry {
    groups: HashMap<String, PackageGroup>,
}

impl GroupRegistry {
    /// Creates a new registry with predefined groups
    pub fn new() -> Self {
        let mut registry = Self {
            groups: HashMap::new(),
        };
        registry.register_default_groups();
        registry
    }

    /// Register a new package group
    pub fn register(&mut self, group: PackageGroup) {
        self.groups.insert(group.id.clone(), group);
    }

    /// Get a group by ID
    pub fn get(&self, id: &str) -> Option<&PackageGroup> {
        self.groups.get(id)
    }

    /// List all available groups
    pub fn list(&self) -> Vec<&PackageGroup> {
        let mut groups: Vec<_> = self.groups.values().collect();
        groups.sort_by(|a, b| a.category.cmp(&b.category).then(a.name.cmp(&b.name)));
        groups
    }

    /// List groups in a specific category
    pub fn list_by_category(&self, category: &str) -> Vec<&PackageGroup> {
        let mut groups: Vec<_> = self
            .groups
            .values()
            .filter(|g| g.category == category)
            .collect();
        groups.sort_by(|a, b| a.name.cmp(&b.name));
        groups
    }

    /// Search groups by name or description
    pub fn search(&self, query: &str) -> Vec<&PackageGroup> {
        let query_lower = query.to_lowercase();
        let mut groups: Vec<_> = self
            .groups
            .values()
            .filter(|g| {
                g.name.to_lowercase().contains(&query_lower)
                    || g.description.to_lowercase().contains(&query_lower)
                    || g.category.to_lowercase().contains(&query_lower)
                    || g.packages
                        .iter()
                        .any(|p| p.to_lowercase().contains(&query_lower))
            })
            .collect();
        groups.sort_by(|a, b| a.name.cmp(&b.name));
        groups
    }

    /// Register all default package groups
    fn register_default_groups(&mut self) {
        // Development Groups
        self.register(
            PackageGroup::new(
                "web-dev",
                "Web Development",
                "Essential tools for modern web development",
                "development",
                vec![
                    "node".to_string(),
                    "yarn".to_string(),
                    "git".to_string(),
                    "docker".to_string(),
                ],
            )
            .with_optional(vec![
                "nginx".to_string(),
                "postgresql".to_string(),
                "redis".to_string(),
            ]),
        );

        self.register(
            PackageGroup::new(
                "rust-dev",
                "Rust Development",
                "Complete Rust development environment",
                "development",
                vec![
                    "rust".to_string(),
                    "cargo".to_string(),
                    "rust-analyzer".to_string(),
                ],
            )
            .with_optional(vec!["llvm".to_string(), "cmake".to_string()]),
        );

        self.register(
            PackageGroup::new(
                "python-dev",
                "Python Development",
                "Python development tools and package managers",
                "development",
                vec![
                    "python".to_string(),
                    "pip".to_string(),
                    "pipenv".to_string(),
                ],
            )
            .with_optional(vec![
                "poetry".to_string(),
                "pyenv".to_string(),
                "black".to_string(),
                "pylint".to_string(),
            ]),
        );

        self.register(
            PackageGroup::new(
                "go-dev",
                "Go Development",
                "Go programming language and tools",
                "development",
                vec!["go".to_string(), "gopls".to_string()],
            )
            .with_optional(vec!["golangci-lint".to_string(), "delve".to_string()]),
        );

        self.register(
            PackageGroup::new(
                "ruby-dev",
                "Ruby Development",
                "Ruby development environment",
                "development",
                vec!["ruby".to_string(), "bundler".to_string()],
            )
            .with_optional(vec!["rbenv".to_string(), "rubocop".to_string()]),
        );

        // DevOps & Infrastructure
        self.register(
            PackageGroup::new(
                "devops",
                "DevOps Tools",
                "Infrastructure and deployment tools",
                "devops",
                vec![
                    "docker".to_string(),
                    "kubectl".to_string(),
                    "terraform".to_string(),
                    "ansible".to_string(),
                ],
            )
            .with_optional(vec![
                "helm".to_string(),
                "aws-cli".to_string(),
                "gcloud".to_string(),
            ]),
        );

        self.register(
            PackageGroup::new(
                "cloud-aws",
                "AWS Tools",
                "Amazon Web Services development tools",
                "devops",
                vec!["aws-cli".to_string(), "aws-sam-cli".to_string()],
            )
            .with_optional(vec!["terraform".to_string(), "localstack".to_string()]),
        );

        // Terminal & CLI Tools
        self.register(
            PackageGroup::new(
                "terminal",
                "Terminal Essentials",
                "Modern terminal tools for productivity",
                "productivity",
                vec![
                    "tmux".to_string(),
                    "fzf".to_string(),
                    "ripgrep".to_string(),
                    "bat".to_string(),
                    "fd".to_string(),
                ],
            )
            .with_optional(vec![
                "starship".to_string(),
                "exa".to_string(),
                "zoxide".to_string(),
                "tldr".to_string(),
            ]),
        );

        self.register(
            PackageGroup::new(
                "shell-power",
                "Power User Shell",
                "Advanced shell and command-line utilities",
                "productivity",
                vec![
                    "zsh".to_string(),
                    "oh-my-zsh".to_string(),
                    "fzf".to_string(),
                    "ripgrep".to_string(),
                ],
            )
            .with_optional(vec!["autojump".to_string(), "thefuck".to_string()]),
        );

        // Editors & IDEs
        self.register(
            PackageGroup::new(
                "vim-complete",
                "Vim Complete Setup",
                "Vim/Neovim with essential plugins",
                "editor",
                vec![
                    "neovim".to_string(),
                    "git".to_string(),
                    "ripgrep".to_string(),
                    "fd".to_string(),
                ],
            )
            .with_optional(vec!["tree-sitter".to_string(), "nodejs".to_string()]),
        );

        // Database Tools
        self.register(
            PackageGroup::new(
                "databases",
                "Database Tools",
                "Common databases and management tools",
                "development",
                vec![
                    "postgresql".to_string(),
                    "redis".to_string(),
                    "sqlite".to_string(),
                ],
            )
            .with_optional(vec!["mongodb".to_string(), "mysql".to_string()]),
        );

        // Multimedia
        self.register(
            PackageGroup::new(
                "media",
                "Multimedia Tools",
                "Audio and video processing utilities",
                "multimedia",
                vec!["ffmpeg".to_string(), "imagemagick".to_string()],
            )
            .with_optional(vec!["youtube-dl".to_string(), "vlc".to_string()]),
        );

        // Security & Privacy
        self.register(
            PackageGroup::new(
                "security",
                "Security Tools",
                "Security and privacy utilities",
                "security",
                vec![
                    "gnupg".to_string(),
                    "openssh".to_string(),
                    "openssl".to_string(),
                ],
            )
            .with_optional(vec!["wireguard".to_string(), "nmap".to_string()]),
        );

        // System Monitoring
        self.register(
            PackageGroup::new(
                "monitoring",
                "System Monitoring",
                "System monitoring and performance tools",
                "system",
                vec!["htop".to_string(), "btop".to_string()],
            )
            .with_optional(vec![
                "glances".to_string(),
                "iotop".to_string(),
                "nethogs".to_string(),
            ]),
        );

        // Git & Version Control
        self.register(
            PackageGroup::new(
                "git-power",
                "Git Power User",
                "Advanced Git tools and utilities",
                "development",
                vec!["git".to_string(), "git-lfs".to_string(), "gh".to_string()],
            )
            .with_optional(vec![
                "lazygit".to_string(),
                "tig".to_string(),
                "delta".to_string(),
            ]),
        );

        // Data Science
        self.register(
            PackageGroup::new(
                "data-science",
                "Data Science",
                "Data analysis and machine learning tools",
                "development",
                vec!["python".to_string(), "jupyter".to_string(), "r".to_string()],
            )
            .with_optional(vec!["octave".to_string(), "gnuplot".to_string()]),
        );
    }
}

impl Default for GroupRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_package_group() {
        let group = PackageGroup::new(
            "test",
            "Test Group",
            "A test group",
            "test",
            vec!["pkg1".to_string(), "pkg2".to_string()],
        );

        assert_eq!(group.id, "test");
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.packages.len(), 2);
        assert_eq!(group.optional_packages.len(), 0);
    }

    #[test]
    fn test_group_with_optional() {
        let group = PackageGroup::new("test", "Test", "Test", "test", vec!["pkg1".to_string()])
            .with_optional(vec!["opt1".to_string(), "opt2".to_string()]);

        assert_eq!(group.packages.len(), 1);
        assert_eq!(group.optional_packages.len(), 2);
        assert_eq!(group.all_packages().len(), 3);
    }

    #[test]
    fn test_registry_initialization() {
        let registry = GroupRegistry::new();
        assert!(registry.groups.len() > 0);
    }

    #[test]
    fn test_get_group() {
        let registry = GroupRegistry::new();
        let group = registry.get("web-dev");
        assert!(group.is_some());
        assert_eq!(group.unwrap().name, "Web Development");
    }

    #[test]
    fn test_list_groups() {
        let registry = GroupRegistry::new();
        let groups = registry.list();
        assert!(groups.len() > 0);
    }

    #[test]
    fn test_list_by_category() {
        let registry = GroupRegistry::new();
        let dev_groups = registry.list_by_category("development");
        assert!(dev_groups.len() > 0);
        assert!(dev_groups.iter().all(|g| g.category == "development"));
    }

    #[test]
    fn test_search_groups() {
        let registry = GroupRegistry::new();
        let results = registry.search("rust");
        assert!(results.len() > 0);
        assert!(results.iter().any(|g| g.id == "rust-dev"));
    }

    #[test]
    fn test_search_by_package_name() {
        let registry = GroupRegistry::new();
        let results = registry.search("docker");
        assert!(results.len() > 0);
    }

    #[test]
    fn test_all_groups_have_packages() {
        let registry = GroupRegistry::new();
        for group in registry.list() {
            assert!(
                !group.packages.is_empty(),
                "Group {} should have at least one package",
                group.id
            );
        }
    }
}
