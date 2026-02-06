use std::collections::{HashMap, HashSet};

/// Represents a package dependency
#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    /// The package name
    pub package: String,
    /// Whether this dependency is required or optional
    pub required: bool,
    /// Human-readable reason for this dependency
    pub reason: String,
}

impl Dependency {
    /// Create a required dependency
    pub fn required(package: &str, reason: &str) -> Self {
        Self {
            package: package.to_string(),
            required: true,
            reason: reason.to_string(),
        }
    }

    /// Create an optional dependency
    pub fn optional(package: &str, reason: &str) -> Self {
        Self {
            package: package.to_string(),
            required: false,
            reason: reason.to_string(),
        }
    }
}

/// Manages package dependencies and relationships
pub struct DependencyGraph {
    /// Map of package name to its dependencies
    dependencies: HashMap<String, Vec<Dependency>>,
}

impl DependencyGraph {
    /// Create a new dependency graph with common package dependencies
    pub fn new() -> Self {
        let mut graph = Self {
            dependencies: HashMap::new(),
        };
        graph.populate_common_dependencies();
        graph
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, package: &str, dependency: Dependency) {
        self.dependencies
            .entry(package.to_string())
            .or_default()
            .push(dependency);
    }

    /// Get all dependencies for a package
    pub fn get_dependencies(&self, package: &str) -> Vec<&Dependency> {
        self.dependencies
            .get(package)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// Check for missing dependencies given a list of installed packages
    pub fn check_missing(&self, packages: &[String]) -> Vec<MissingDependency> {
        let installed: HashSet<String> = packages.iter().cloned().collect();
        let mut missing = Vec::new();

        for package in packages {
            if let Some(deps) = self.dependencies.get(package) {
                for dep in deps {
                    if !installed.contains(&dep.package) {
                        missing.push(MissingDependency {
                            for_package: package.clone(),
                            dependency: dep.clone(),
                        });
                    }
                }
            }
        }

        missing
    }

    /// Get suggested packages based on what's already installed
    pub fn get_suggestions(&self, packages: &[String]) -> Vec<Suggestion> {
        let installed: HashSet<String> = packages.iter().cloned().collect();
        let mut suggestions = Vec::new();

        for package in packages {
            if let Some(deps) = self.dependencies.get(package) {
                for dep in deps {
                    if !dep.required && !installed.contains(&dep.package) {
                        suggestions.push(Suggestion {
                            package: dep.package.clone(),
                            reason: format!("Works well with {} - {}", package, dep.reason),
                            related_to: package.clone(),
                        });
                    }
                }
            }
        }

        // Deduplicate suggestions
        suggestions.sort_by(|a, b| a.package.cmp(&b.package));
        suggestions.dedup_by(|a, b| a.package == b.package);

        suggestions
    }

    /// Populate the graph with common package dependencies
    fn populate_common_dependencies(&mut self) {
        // Editor dependencies
        self.add_dependency(
            "neovim",
            Dependency::required("git", "required for plugin management"),
        );
        self.add_dependency(
            "neovim",
            Dependency::optional("ripgrep", "enhanced search capabilities"),
        );
        self.add_dependency(
            "neovim",
            Dependency::optional("fzf", "fuzzy finding in files"),
        );
        self.add_dependency("neovim", Dependency::optional("fd", "faster file finding"));

        self.add_dependency(
            "vim",
            Dependency::optional("git", "version control integration"),
        );

        // Docker ecosystem
        self.add_dependency(
            "docker",
            Dependency::optional("docker-compose", "multi-container orchestration"),
        );
        self.add_dependency(
            "docker",
            Dependency::optional("kubectl", "if using Kubernetes"),
        );

        // Kubernetes tools
        self.add_dependency(
            "kubectl",
            Dependency::optional("helm", "Kubernetes package manager"),
        );
        self.add_dependency(
            "kubectl",
            Dependency::optional("k9s", "terminal UI for Kubernetes"),
        );

        // Development tools
        self.add_dependency(
            "node",
            Dependency::optional("yarn", "alternative package manager"),
        );
        self.add_dependency("node", Dependency::optional("nvm", "Node version manager"));

        self.add_dependency(
            "python",
            Dependency::optional("pipenv", "dependency management"),
        );
        self.add_dependency(
            "python",
            Dependency::optional("pyenv", "Python version manager"),
        );

        self.add_dependency("go", Dependency::optional("gopls", "Go language server"));

        // Rust ecosystem
        self.add_dependency(
            "rust",
            Dependency::optional("rust-analyzer", "Rust language server"),
        );
        self.add_dependency(
            "cargo",
            Dependency::optional("cargo-watch", "watch for changes and rebuild"),
        );
        self.add_dependency(
            "cargo",
            Dependency::optional("cargo-edit", "manage dependencies from CLI"),
        );

        // Terminal tools that work well together
        self.add_dependency(
            "ripgrep",
            Dependency::optional("bat", "syntax highlighting for rg output"),
        );
        self.add_dependency(
            "ripgrep",
            Dependency::optional("fzf", "combine with fuzzy finding"),
        );

        self.add_dependency(
            "fzf",
            Dependency::optional("fd", "faster alternative to find"),
        );
        self.add_dependency(
            "fzf",
            Dependency::optional("bat", "preview files with syntax highlighting"),
        );

        self.add_dependency(
            "bat",
            Dependency::optional("git", "show git diffs with syntax highlighting"),
        );

        // Git ecosystem
        self.add_dependency(
            "git",
            Dependency::optional("gh", "GitHub CLI for PR management"),
        );
        self.add_dependency(
            "git",
            Dependency::optional("delta", "better git diff viewer"),
        );
        self.add_dependency(
            "git",
            Dependency::optional("lazygit", "terminal UI for git"),
        );

        // tmux ecosystem
        self.add_dependency("tmux", Dependency::optional("fzf", "fuzzy finding in tmux"));

        // Infrastructure tools
        self.add_dependency(
            "terraform",
            Dependency::optional("tflint", "Terraform linter"),
        );
        self.add_dependency(
            "terraform",
            Dependency::optional("terragrunt", "Terraform wrapper"),
        );

        self.add_dependency(
            "ansible",
            Dependency::optional("ansible-lint", "Ansible linter"),
        );

        // Data science
        self.add_dependency(
            "jupyter",
            Dependency::required("python", "Jupyter runs on Python"),
        );
        self.add_dependency(
            "jupyterlab",
            Dependency::required("python", "JupyterLab runs on Python"),
        );

        // Build tools
        self.add_dependency(
            "cmake",
            Dependency::optional("ninja", "faster build system"),
        );
        self.add_dependency(
            "cmake",
            Dependency::optional("ccache", "speed up C/C++ compilation"),
        );

        // Shell enhancements
        self.add_dependency(
            "zsh",
            Dependency::optional("starship", "fast, customizable prompt"),
        );
        self.add_dependency(
            "bash",
            Dependency::optional("starship", "fast, customizable prompt"),
        );

        // macOS specific
        self.add_dependency(
            "iterm2",
            Dependency::optional("tmux", "terminal multiplexing"),
        );
        self.add_dependency(
            "iterm2",
            Dependency::optional("zsh", "better shell than bash"),
        );

        // Database tools
        self.add_dependency(
            "postgresql",
            Dependency::optional("pgcli", "better PostgreSQL CLI"),
        );
        self.add_dependency("mysql", Dependency::optional("mycli", "better MySQL CLI"));
        self.add_dependency(
            "redis",
            Dependency::optional("redis-cli", "Redis command-line client"),
        );
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a missing dependency
#[derive(Debug, Clone)]
pub struct MissingDependency {
    /// The package that needs this dependency
    pub for_package: String,
    /// The missing dependency
    pub dependency: Dependency,
}

impl MissingDependency {
    /// Format as a user-friendly message
    pub fn format_message(&self) -> String {
        if self.dependency.required {
            format!(
                "⚠ '{}' requires '{}' ({})",
                self.for_package, self.dependency.package, self.dependency.reason
            )
        } else {
            format!(
                "ℹ '{}' works better with '{}' ({})",
                self.for_package, self.dependency.package, self.dependency.reason
            )
        }
    }

    /// Check if this is a required dependency
    pub fn is_required(&self) -> bool {
        self.dependency.required
    }
}

/// Represents a suggested package
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// The suggested package name
    pub package: String,
    /// Why this is suggested
    pub reason: String,
    /// The package that triggered this suggestion
    pub related_to: String,
}

impl Suggestion {
    /// Format as a user-friendly message
    pub fn format_message(&self) -> String {
        format!("💡 Consider '{}': {}", self.package, self.reason)
    }
}

/// Helper to analyze a package list and provide recommendations
pub struct DependencyAnalyzer {
    graph: DependencyGraph,
}

impl DependencyAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }

    /// Analyze a list of packages and return recommendations
    pub fn analyze(&self, packages: &[String]) -> AnalysisResult {
        let missing = self.graph.check_missing(packages);
        let suggestions = self.graph.get_suggestions(packages);

        // Separate required from optional
        let (required, optional): (Vec<_>, Vec<_>) =
            missing.into_iter().partition(|m| m.dependency.required);

        AnalysisResult {
            required_missing: required,
            optional_missing: optional,
            suggestions,
        }
    }

    /// Get the dependency graph
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of dependency analysis
#[derive(Debug)]
pub struct AnalysisResult {
    /// Required dependencies that are missing
    pub required_missing: Vec<MissingDependency>,
    /// Optional dependencies that are missing
    pub optional_missing: Vec<MissingDependency>,
    /// Suggested packages based on what's installed
    pub suggestions: Vec<Suggestion>,
}

impl AnalysisResult {
    /// Check if there are any required missing dependencies
    pub fn has_required_missing(&self) -> bool {
        !self.required_missing.is_empty()
    }

    /// Check if there are any optional missing dependencies
    pub fn has_optional_missing(&self) -> bool {
        !self.optional_missing.is_empty()
    }

    /// Check if there are any suggestions
    pub fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }

    /// Get all missing packages (required + optional)
    pub fn all_missing_packages(&self) -> Vec<String> {
        let mut packages: Vec<String> = self
            .required_missing
            .iter()
            .chain(self.optional_missing.iter())
            .map(|m| m.dependency.package.clone())
            .collect();

        packages.sort();
        packages.dedup();
        packages
    }

    /// Format a summary of the analysis
    pub fn format_summary(&self) -> String {
        let mut lines = Vec::new();

        if self.has_required_missing() {
            lines.push(format!(
                "{} required dependencies missing",
                self.required_missing.len()
            ));
        }

        if self.has_optional_missing() {
            lines.push(format!(
                "{} optional dependencies could improve your setup",
                self.optional_missing.len()
            ));
        }

        if self.has_suggestions() {
            lines.push(format!(
                "{} package suggestions based on your current setup",
                self.suggestions.len()
            ));
        }

        if lines.is_empty() {
            "All dependencies satisfied!".to_string()
        } else {
            lines.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_creation() {
        let required = Dependency::required("git", "for version control");
        assert!(required.required);
        assert_eq!(required.package, "git");

        let optional = Dependency::optional("fzf", "for fuzzy finding");
        assert!(!optional.required);
        assert_eq!(optional.package, "fzf");
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("test", Dependency::required("dep", "test dependency"));

        let deps = graph.get_dependencies("test");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].package, "dep");
    }

    #[test]
    fn test_neovim_dependencies() {
        let graph = DependencyGraph::new();
        let deps = graph.get_dependencies("neovim");

        // Should have git as required
        let git_dep = deps.iter().find(|d| d.package == "git");
        assert!(git_dep.is_some());
        assert!(git_dep.unwrap().required);

        // Should have ripgrep as optional
        let rg_dep = deps.iter().find(|d| d.package == "ripgrep");
        assert!(rg_dep.is_some());
        assert!(!rg_dep.unwrap().required);
    }

    #[test]
    fn test_check_missing_required() {
        let graph = DependencyGraph::new();
        let packages = vec!["neovim".to_string()]; // Has neovim but not git

        let missing = graph.check_missing(&packages);

        // Should find git as missing (required by neovim)
        let git_missing = missing.iter().find(|m| m.dependency.package == "git");
        assert!(git_missing.is_some());
        assert!(git_missing.unwrap().dependency.required);
    }

    #[test]
    fn test_check_missing_with_deps_installed() {
        let graph = DependencyGraph::new();
        let packages = vec!["neovim".to_string(), "git".to_string()];

        let missing = graph.check_missing(&packages);

        // Git is installed, so should not be in missing
        let git_missing = missing.iter().find(|m| m.dependency.package == "git");
        assert!(git_missing.is_none());
    }

    #[test]
    fn test_get_suggestions() {
        let graph = DependencyGraph::new();
        let packages = vec!["neovim".to_string(), "git".to_string()];

        let suggestions = graph.get_suggestions(&packages);

        // Should suggest ripgrep, fzf, fd for neovim
        assert!(!suggestions.is_empty());

        let has_ripgrep = suggestions.iter().any(|s| s.package == "ripgrep");
        let has_fzf = suggestions.iter().any(|s| s.package == "fzf");

        assert!(has_ripgrep || has_fzf);
    }

    #[test]
    fn test_docker_dependencies() {
        let graph = DependencyGraph::new();
        let deps = graph.get_dependencies("docker");

        // Should suggest docker-compose
        let compose = deps.iter().find(|d| d.package == "docker-compose");
        assert!(compose.is_some());
        assert!(!compose.unwrap().required); // Optional
    }

    #[test]
    fn test_dependency_analyzer() {
        let analyzer = DependencyAnalyzer::new();
        let packages = vec!["neovim".to_string()];

        let result = analyzer.analyze(&packages);

        // Should have git as required missing
        assert!(result.has_required_missing());
        assert_eq!(result.required_missing.len(), 1);
        assert_eq!(result.required_missing[0].dependency.package, "git");
    }

    #[test]
    fn test_analysis_result_all_missing() {
        let analyzer = DependencyAnalyzer::new();
        let packages = vec!["neovim".to_string()];

        let result = analyzer.analyze(&packages);
        let all_missing = result.all_missing_packages();

        assert!(!all_missing.is_empty());
        assert!(all_missing.contains(&"git".to_string()));
    }

    #[test]
    fn test_missing_dependency_format() {
        let missing = MissingDependency {
            for_package: "neovim".to_string(),
            dependency: Dependency::required("git", "for plugin management"),
        };

        let message = missing.format_message();
        assert!(message.contains("neovim"));
        assert!(message.contains("git"));
        assert!(message.contains("plugin management"));
    }

    #[test]
    fn test_suggestion_format() {
        let suggestion = Suggestion {
            package: "ripgrep".to_string(),
            reason: "fast search".to_string(),
            related_to: "neovim".to_string(),
        };

        let message = suggestion.format_message();
        assert!(message.contains("ripgrep"));
        assert!(message.contains("fast search"));
    }

    #[test]
    fn test_analysis_result_summary() {
        let result = AnalysisResult {
            required_missing: vec![MissingDependency {
                for_package: "neovim".to_string(),
                dependency: Dependency::required("git", "test"),
            }],
            optional_missing: vec![],
            suggestions: vec![],
        };

        let summary = result.format_summary();
        assert!(summary.contains("1 required"));
    }

    #[test]
    fn test_no_missing_dependencies() {
        let analyzer = DependencyAnalyzer::new();
        let packages = vec!["neovim".to_string(), "git".to_string()];

        let result = analyzer.analyze(&packages);

        // Git is installed, no required missing
        assert!(!result.has_required_missing());
    }
}
