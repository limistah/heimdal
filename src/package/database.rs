use std::collections::HashMap;

/// Package metadata information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name (canonical/common name)
    pub name: String,
    /// Short description of what the package does
    pub description: String,
    /// Category for organization
    pub category: PackageCategory,
    /// Popularity score (0-100, higher = more popular)
    pub popularity: u8,
    /// Alternative/similar packages
    pub alternatives: Vec<String>,
    /// Related packages that work well together
    pub related: Vec<String>,
    /// Package tags for filtering
    pub tags: Vec<String>,
}

/// Package categories for organization
#[derive(Debug, Clone, PartialEq)]
pub enum PackageCategory {
    /// Essential tools (git, vim, curl, etc.)
    Essential,
    /// Development tools (compilers, build tools, etc.)
    Development,
    /// Terminal utilities (tmux, fzf, ripgrep, etc.)
    Terminal,
    /// Text editors and IDEs
    Editor,
    /// Programming language tools
    Language,
    /// Container and orchestration tools
    Container,
    /// DevOps and infrastructure
    Infrastructure,
    /// Database tools
    Database,
    /// Network utilities
    Network,
    /// GUI applications
    Application,
    /// Data science and ML
    DataScience,
    /// Documentation and writing
    Documentation,
    /// Other/Miscellaneous
    Other,
}

impl PackageCategory {
    pub fn as_str(&self) -> &str {
        match self {
            PackageCategory::Essential => "Essential",
            PackageCategory::Development => "Development",
            PackageCategory::Terminal => "Terminal",
            PackageCategory::Editor => "Editor",
            PackageCategory::Language => "Language",
            PackageCategory::Container => "Container",
            PackageCategory::Infrastructure => "Infrastructure",
            PackageCategory::Database => "Database",
            PackageCategory::Network => "Network",
            PackageCategory::Application => "Application",
            PackageCategory::DataScience => "Data Science",
            PackageCategory::Documentation => "Documentation",
            PackageCategory::Other => "Other",
        }
    }
}

/// Database of package metadata
pub struct PackageDatabase {
    packages: HashMap<String, PackageInfo>,
}

impl PackageDatabase {
    /// Create a new package database with common packages
    pub fn new() -> Self {
        let mut db = Self {
            packages: HashMap::new(),
        };
        db.populate_packages();
        db
    }

    /// Add a package to the database
    pub fn add(&mut self, info: PackageInfo) {
        self.packages.insert(info.name.clone(), info);
    }

    /// Get package information by name
    pub fn get(&self, name: &str) -> Option<&PackageInfo> {
        self.packages.get(name)
    }

    /// Check if a package exists in the database
    pub fn contains(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    /// Search packages by name or description (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<&PackageInfo> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&PackageInfo> = self
            .packages
            .values()
            .filter(|pkg| {
                pkg.name.to_lowercase().contains(&query_lower)
                    || pkg.description.to_lowercase().contains(&query_lower)
                    || pkg
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect();

        // Sort by popularity (descending)
        results.sort_by(|a, b| b.popularity.cmp(&a.popularity));
        results
    }

    /// Get packages by category
    pub fn by_category(&self, category: &PackageCategory) -> Vec<&PackageInfo> {
        let mut results: Vec<&PackageInfo> = self
            .packages
            .values()
            .filter(|pkg| &pkg.category == category)
            .collect();

        results.sort_by(|a, b| b.popularity.cmp(&a.popularity));
        results
    }

    /// Get the most popular packages (top N)
    pub fn popular(&self, limit: usize) -> Vec<&PackageInfo> {
        let mut results: Vec<&PackageInfo> = self.packages.values().collect();
        results.sort_by(|a, b| b.popularity.cmp(&a.popularity));
        results.into_iter().take(limit).collect()
    }

    /// Get alternative packages for a given package
    pub fn get_alternatives(&self, package: &str) -> Vec<&PackageInfo> {
        if let Some(pkg) = self.get(package) {
            pkg.alternatives
                .iter()
                .filter_map(|alt| self.get(alt))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get related packages for a given package
    pub fn get_related(&self, package: &str) -> Vec<&PackageInfo> {
        if let Some(pkg) = self.get(package) {
            pkg.related.iter().filter_map(|rel| self.get(rel)).collect()
        } else {
            Vec::new()
        }
    }

    /// Get all packages as a vec
    pub fn all(&self) -> Vec<&PackageInfo> {
        self.packages.values().collect()
    }

    /// Populate the database with common packages
    fn populate_packages(&mut self) {
        // Essential tools
        self.add(PackageInfo {
            name: "git".to_string(),
            description: "Distributed version control system".to_string(),
            category: PackageCategory::Essential,
            popularity: 100,
            alternatives: vec![],
            related: vec!["gh".to_string(), "delta".to_string(), "lazygit".to_string()],
            tags: vec!["vcs".to_string(), "version-control".to_string()],
        });

        self.add(PackageInfo {
            name: "curl".to_string(),
            description: "Command-line tool for transferring data with URLs".to_string(),
            category: PackageCategory::Network,
            popularity: 95,
            alternatives: vec!["wget".to_string(), "httpie".to_string()],
            related: vec!["jq".to_string()],
            tags: vec!["http".to_string(), "network".to_string()],
        });

        self.add(PackageInfo {
            name: "wget".to_string(),
            description: "Network downloader supporting HTTP, HTTPS, FTP".to_string(),
            category: PackageCategory::Network,
            popularity: 85,
            alternatives: vec!["curl".to_string()],
            related: vec![],
            tags: vec!["http".to_string(), "download".to_string()],
        });

        // Editors
        self.add(PackageInfo {
            name: "neovim".to_string(),
            description: "Hyperextensible Vim-based text editor".to_string(),
            category: PackageCategory::Editor,
            popularity: 90,
            alternatives: vec!["vim".to_string(), "emacs".to_string()],
            related: vec!["ripgrep".to_string(), "fzf".to_string(), "fd".to_string()],
            tags: vec![
                "editor".to_string(),
                "vim".to_string(),
                "terminal".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "vim".to_string(),
            description: "Highly configurable text editor".to_string(),
            category: PackageCategory::Editor,
            popularity: 85,
            alternatives: vec!["neovim".to_string(), "emacs".to_string()],
            related: vec![],
            tags: vec!["editor".to_string(), "terminal".to_string()],
        });

        // Terminal tools
        self.add(PackageInfo {
            name: "tmux".to_string(),
            description: "Terminal multiplexer for managing multiple terminal sessions".to_string(),
            category: PackageCategory::Terminal,
            popularity: 88,
            alternatives: vec!["screen".to_string(), "zellij".to_string()],
            related: vec!["fzf".to_string()],
            tags: vec!["terminal".to_string(), "multiplexer".to_string()],
        });

        self.add(PackageInfo {
            name: "ripgrep".to_string(),
            description: "Fast line-oriented search tool (better grep)".to_string(),
            category: PackageCategory::Terminal,
            popularity: 92,
            alternatives: vec!["grep".to_string(), "ag".to_string()],
            related: vec!["fzf".to_string(), "bat".to_string(), "fd".to_string()],
            tags: vec!["search".to_string(), "grep".to_string(), "fast".to_string()],
        });

        self.add(PackageInfo {
            name: "bat".to_string(),
            description: "Cat clone with syntax highlighting and Git integration".to_string(),
            category: PackageCategory::Terminal,
            popularity: 88,
            alternatives: vec!["cat".to_string()],
            related: vec!["ripgrep".to_string(), "fd".to_string()],
            tags: vec![
                "cat".to_string(),
                "syntax".to_string(),
                "highlighting".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "fd".to_string(),
            description: "Fast and user-friendly alternative to find".to_string(),
            category: PackageCategory::Terminal,
            popularity: 86,
            alternatives: vec!["find".to_string()],
            related: vec!["ripgrep".to_string(), "fzf".to_string()],
            tags: vec!["find".to_string(), "search".to_string(), "fast".to_string()],
        });

        self.add(PackageInfo {
            name: "fzf".to_string(),
            description: "Command-line fuzzy finder".to_string(),
            category: PackageCategory::Terminal,
            popularity: 94,
            alternatives: vec!["skim".to_string()],
            related: vec!["ripgrep".to_string(), "fd".to_string(), "bat".to_string()],
            tags: vec![
                "fuzzy".to_string(),
                "finder".to_string(),
                "search".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "jq".to_string(),
            description: "Command-line JSON processor".to_string(),
            category: PackageCategory::Terminal,
            popularity: 90,
            alternatives: vec!["yq".to_string()],
            related: vec!["curl".to_string()],
            tags: vec!["json".to_string(), "parser".to_string()],
        });

        self.add(PackageInfo {
            name: "htop".to_string(),
            description: "Interactive process viewer (better top)".to_string(),
            category: PackageCategory::Terminal,
            popularity: 87,
            alternatives: vec!["top".to_string(), "btop".to_string(), "glances".to_string()],
            related: vec![],
            tags: vec!["monitoring".to_string(), "processes".to_string()],
        });

        self.add(PackageInfo {
            name: "tree".to_string(),
            description: "Display directory tree structure".to_string(),
            category: PackageCategory::Terminal,
            popularity: 80,
            alternatives: vec!["exa".to_string()],
            related: vec![],
            tags: vec!["directory".to_string(), "tree".to_string()],
        });

        // Programming languages
        self.add(PackageInfo {
            name: "node".to_string(),
            description: "JavaScript runtime built on V8 engine".to_string(),
            category: PackageCategory::Language,
            popularity: 95,
            alternatives: vec!["bun".to_string(), "deno".to_string()],
            related: vec!["yarn".to_string(), "npm".to_string()],
            tags: vec![
                "javascript".to_string(),
                "js".to_string(),
                "runtime".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "python".to_string(),
            description: "Interpreted high-level programming language".to_string(),
            category: PackageCategory::Language,
            popularity: 98,
            alternatives: vec![],
            related: vec!["pip".to_string(), "pipenv".to_string(), "pyenv".to_string()],
            tags: vec!["python".to_string(), "programming".to_string()],
        });

        self.add(PackageInfo {
            name: "go".to_string(),
            description: "Go programming language compiler".to_string(),
            category: PackageCategory::Language,
            popularity: 92,
            alternatives: vec![],
            related: vec!["gopls".to_string()],
            tags: vec![
                "golang".to_string(),
                "go".to_string(),
                "programming".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "rust".to_string(),
            description: "Systems programming language focused on safety".to_string(),
            category: PackageCategory::Language,
            popularity: 90,
            alternatives: vec![],
            related: vec!["cargo".to_string(), "rust-analyzer".to_string()],
            tags: vec![
                "rust".to_string(),
                "systems".to_string(),
                "programming".to_string(),
            ],
        });

        // Containers & Orchestration
        self.add(PackageInfo {
            name: "docker".to_string(),
            description: "Platform for developing, shipping, and running containers".to_string(),
            category: PackageCategory::Container,
            popularity: 96,
            alternatives: vec!["podman".to_string()],
            related: vec!["docker-compose".to_string(), "kubectl".to_string()],
            tags: vec!["container".to_string(), "devops".to_string()],
        });

        self.add(PackageInfo {
            name: "docker-compose".to_string(),
            description: "Tool for defining and running multi-container Docker applications"
                .to_string(),
            category: PackageCategory::Container,
            popularity: 92,
            alternatives: vec![],
            related: vec!["docker".to_string()],
            tags: vec!["container".to_string(), "orchestration".to_string()],
        });

        self.add(PackageInfo {
            name: "kubectl".to_string(),
            description: "Kubernetes command-line tool".to_string(),
            category: PackageCategory::Container,
            popularity: 90,
            alternatives: vec![],
            related: vec!["helm".to_string(), "k9s".to_string()],
            tags: vec![
                "kubernetes".to_string(),
                "k8s".to_string(),
                "orchestration".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "helm".to_string(),
            description: "Kubernetes package manager".to_string(),
            category: PackageCategory::Container,
            popularity: 85,
            alternatives: vec![],
            related: vec!["kubectl".to_string()],
            tags: vec!["kubernetes".to_string(), "package-manager".to_string()],
        });

        self.add(PackageInfo {
            name: "k9s".to_string(),
            description: "Terminal UI for managing Kubernetes clusters".to_string(),
            category: PackageCategory::Container,
            popularity: 83,
            alternatives: vec![],
            related: vec!["kubectl".to_string()],
            tags: vec![
                "kubernetes".to_string(),
                "ui".to_string(),
                "terminal".to_string(),
            ],
        });

        // Infrastructure
        self.add(PackageInfo {
            name: "terraform".to_string(),
            description: "Infrastructure as code software tool".to_string(),
            category: PackageCategory::Infrastructure,
            popularity: 93,
            alternatives: vec!["pulumi".to_string(), "cloudformation".to_string()],
            related: vec!["tflint".to_string()],
            tags: vec![
                "iac".to_string(),
                "infrastructure".to_string(),
                "devops".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "ansible".to_string(),
            description: "IT automation tool for configuration management".to_string(),
            category: PackageCategory::Infrastructure,
            popularity: 88,
            alternatives: vec!["puppet".to_string(), "chef".to_string()],
            related: vec![],
            tags: vec!["automation".to_string(), "configuration".to_string()],
        });

        // Git tools
        self.add(PackageInfo {
            name: "gh".to_string(),
            description: "GitHub CLI for managing pull requests and issues".to_string(),
            category: PackageCategory::Development,
            popularity: 87,
            alternatives: vec!["hub".to_string()],
            related: vec!["git".to_string()],
            tags: vec!["github".to_string(), "git".to_string(), "cli".to_string()],
        });

        self.add(PackageInfo {
            name: "delta".to_string(),
            description: "Syntax-highlighting pager for git and diff output".to_string(),
            category: PackageCategory::Development,
            popularity: 82,
            alternatives: vec!["diff-so-fancy".to_string()],
            related: vec!["git".to_string()],
            tags: vec!["git".to_string(), "diff".to_string(), "syntax".to_string()],
        });

        self.add(PackageInfo {
            name: "lazygit".to_string(),
            description: "Simple terminal UI for git commands".to_string(),
            category: PackageCategory::Development,
            popularity: 84,
            alternatives: vec!["tig".to_string()],
            related: vec!["git".to_string()],
            tags: vec!["git".to_string(), "ui".to_string(), "terminal".to_string()],
        });

        // Build tools
        self.add(PackageInfo {
            name: "make".to_string(),
            description: "Build automation tool".to_string(),
            category: PackageCategory::Development,
            popularity: 85,
            alternatives: vec!["cmake".to_string(), "ninja".to_string()],
            related: vec![],
            tags: vec!["build".to_string(), "automation".to_string()],
        });

        self.add(PackageInfo {
            name: "cmake".to_string(),
            description: "Cross-platform build system generator".to_string(),
            category: PackageCategory::Development,
            popularity: 83,
            alternatives: vec!["make".to_string()],
            related: vec!["ninja".to_string()],
            tags: vec!["build".to_string(), "c++".to_string()],
        });

        // Databases
        self.add(PackageInfo {
            name: "postgresql".to_string(),
            description: "Powerful, open-source object-relational database".to_string(),
            category: PackageCategory::Database,
            popularity: 91,
            alternatives: vec!["mysql".to_string(), "sqlite".to_string()],
            related: vec!["pgcli".to_string()],
            tags: vec![
                "database".to_string(),
                "sql".to_string(),
                "postgres".to_string(),
            ],
        });

        self.add(PackageInfo {
            name: "redis".to_string(),
            description: "In-memory data structure store".to_string(),
            category: PackageCategory::Database,
            popularity: 89,
            alternatives: vec!["memcached".to_string()],
            related: vec![],
            tags: vec![
                "cache".to_string(),
                "database".to_string(),
                "nosql".to_string(),
            ],
        });

        // Package managers
        self.add(PackageInfo {
            name: "yarn".to_string(),
            description: "Fast, reliable JavaScript package manager".to_string(),
            category: PackageCategory::Development,
            popularity: 86,
            alternatives: vec!["npm".to_string(), "pnpm".to_string()],
            related: vec!["node".to_string()],
            tags: vec!["javascript".to_string(), "package-manager".to_string()],
        });

        self.add(PackageInfo {
            name: "pipenv".to_string(),
            description: "Python packaging tool with dependency management".to_string(),
            category: PackageCategory::Development,
            popularity: 80,
            alternatives: vec!["poetry".to_string(), "pip".to_string()],
            related: vec!["python".to_string()],
            tags: vec!["python".to_string(), "package-manager".to_string()],
        });

        // Shell enhancements
        self.add(PackageInfo {
            name: "zsh".to_string(),
            description: "Powerful shell with scripting support".to_string(),
            category: PackageCategory::Terminal,
            popularity: 89,
            alternatives: vec!["bash".to_string(), "fish".to_string()],
            related: vec!["starship".to_string()],
            tags: vec!["shell".to_string(), "terminal".to_string()],
        });

        self.add(PackageInfo {
            name: "starship".to_string(),
            description: "Fast, customizable shell prompt".to_string(),
            category: PackageCategory::Terminal,
            popularity: 87,
            alternatives: vec!["powerlevel10k".to_string()],
            related: vec!["zsh".to_string()],
            tags: vec!["prompt".to_string(), "shell".to_string()],
        });

        // Documentation
        self.add(PackageInfo {
            name: "pandoc".to_string(),
            description: "Universal document converter".to_string(),
            category: PackageCategory::Documentation,
            popularity: 82,
            alternatives: vec![],
            related: vec![],
            tags: vec![
                "markdown".to_string(),
                "converter".to_string(),
                "documentation".to_string(),
            ],
        });
    }
}

impl Default for PackageDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = PackageDatabase::new();
        assert!(db.contains("git"));
        assert!(db.contains("neovim"));
        assert!(db.contains("docker"));
    }

    #[test]
    fn test_get_package() {
        let db = PackageDatabase::new();
        let git = db.get("git");
        assert!(git.is_some());

        let git = git.unwrap();
        assert_eq!(git.name, "git");
        assert_eq!(git.popularity, 100);
        assert_eq!(git.category, PackageCategory::Essential);
    }

    #[test]
    fn test_search_by_name() {
        let db = PackageDatabase::new();
        let results = db.search("git");

        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p.name == "git"));
        assert!(results.iter().any(|p| p.name == "lazygit"));
    }

    #[test]
    fn test_search_by_description() {
        let db = PackageDatabase::new();
        let results = db.search("fuzzy");

        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p.name == "fzf"));
    }

    #[test]
    fn test_search_by_tag() {
        let db = PackageDatabase::new();
        let results = db.search("kubernetes");

        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p.name == "kubectl"));
        assert!(results.iter().any(|p| p.name == "helm"));
    }

    #[test]
    fn test_by_category() {
        let db = PackageDatabase::new();
        let editors = db.by_category(&PackageCategory::Editor);

        assert!(!editors.is_empty());
        assert!(editors.iter().any(|p| p.name == "neovim"));
        assert!(editors.iter().any(|p| p.name == "vim"));
    }

    #[test]
    fn test_popular_packages() {
        let db = PackageDatabase::new();
        let popular = db.popular(5);

        assert_eq!(popular.len(), 5);
        // Should include high-popularity packages
        assert!(popular.iter().any(|p| p.name == "git"));
    }

    #[test]
    fn test_get_alternatives() {
        let db = PackageDatabase::new();
        let alternatives = db.get_alternatives("neovim");

        assert!(!alternatives.is_empty());
        assert!(alternatives.iter().any(|p| p.name == "vim"));
    }

    #[test]
    fn test_get_related() {
        let db = PackageDatabase::new();
        let related = db.get_related("neovim");

        assert!(!related.is_empty());
        assert!(related
            .iter()
            .any(|p| p.name == "ripgrep" || p.name == "fzf"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let db = PackageDatabase::new();
        let results_lower = db.search("docker");
        let results_upper = db.search("DOCKER");

        assert_eq!(results_lower.len(), results_upper.len());
    }

    #[test]
    fn test_package_info_fields() {
        let db = PackageDatabase::new();
        let docker = db.get("docker").unwrap();

        assert!(!docker.description.is_empty());
        assert!(!docker.tags.is_empty());
        assert!(docker.popularity > 0);
    }

    #[test]
    fn test_all_packages() {
        let db = PackageDatabase::new();
        let all = db.all();

        assert!(all.len() > 30); // Should have many packages
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(PackageCategory::Essential.as_str(), "Essential");
        assert_eq!(PackageCategory::Terminal.as_str(), "Terminal");
        assert_eq!(PackageCategory::Container.as_str(), "Container");
    }
}
