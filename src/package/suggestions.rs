use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

use super::database::{PackageDatabase, PackageInfo};

/// A package suggestion with context
#[derive(Debug, Clone)]
pub struct PackageSuggestion<'a> {
    /// The suggested package
    pub package: &'a PackageInfo,
    /// Reason for suggestion
    pub reason: String,
    /// Relevance score (0-100, higher = more relevant)
    pub relevance: u8,
    /// Detected files that triggered this suggestion
    pub detected_files: Vec<String>,
}

/// Tool detection result
#[derive(Debug, Clone)]
pub struct DetectedTool {
    /// Tool name (e.g., "Node.js", "Rust", "Python")
    pub name: String,
    /// File patterns used for detection (e.g., "package.json", "Cargo.toml")
    pub patterns: Vec<String>,
    /// Actual detected file paths that matched the patterns
    pub detected_files: Vec<String>,
    /// Suggested packages
    pub packages: Vec<String>,
}

/// Package suggestion engine
pub struct SuggestionEngine {
    db: PackageDatabase,
    /// Map of file patterns to tool info
    tool_patterns: HashMap<String, DetectedTool>,
}

impl SuggestionEngine {
    /// Create a new suggestion engine
    pub fn new() -> Self {
        let mut engine = Self {
            db: PackageDatabase::new(),
            tool_patterns: HashMap::new(),
        };
        engine.initialize_patterns();
        engine
    }

    /// Initialize file pattern mappings
    fn initialize_patterns(&mut self) {
        // Node.js ecosystem
        self.add_tool(
            "Node.js",
            vec!["package.json", "yarn.lock", "pnpm-lock.yaml"],
            vec!["node", "yarn", "npm"],
        );

        // Rust
        self.add_tool(
            "Rust",
            vec!["Cargo.toml", "Cargo.lock"],
            vec!["rust", "cargo"],
        );

        // Python
        self.add_tool(
            "Python",
            vec![
                "requirements.txt",
                "setup.py",
                "pyproject.toml",
                "Pipfile",
                "poetry.lock",
            ],
            vec!["python", "pip", "pipenv"],
        );

        // Go
        self.add_tool("Go", vec!["go.mod", "go.sum"], vec!["go"]);

        // Ruby
        self.add_tool(
            "Ruby",
            vec!["Gemfile", "Gemfile.lock", ".ruby-version"],
            vec!["ruby"],
        );

        // Docker
        self.add_tool(
            "Docker",
            vec![
                "Dockerfile",
                "docker-compose.yml",
                "docker-compose.yaml",
                ".dockerignore",
            ],
            vec!["docker", "docker-compose"],
        );

        // Kubernetes
        self.add_tool(
            "Kubernetes",
            vec!["k8s.yaml", "k8s.yml", "kustomization.yaml"],
            vec!["kubectl", "helm", "k9s"],
        );

        // Terraform
        self.add_tool(
            "Terraform",
            vec!["*.tf", "terraform.tfvars"],
            vec!["terraform"],
        );

        // Git
        self.add_tool(
            "Git",
            vec![".gitignore", ".gitattributes"],
            vec!["git", "gh", "delta", "lazygit"],
        );

        // Vim/Neovim
        self.add_tool(
            "Vim/Neovim",
            vec![".vimrc", "init.vim", "init.lua"],
            vec!["neovim", "vim"],
        );

        // Tmux
        self.add_tool("Tmux", vec![".tmux.conf"], vec!["tmux"]);

        // Shell
        self.add_tool(
            "Zsh",
            vec![".zshrc", ".zsh_history"],
            vec!["zsh", "starship"],
        );
        self.add_tool("Bash", vec![".bashrc", ".bash_profile"], vec!["starship"]);

        // Database tools
        self.add_tool(
            "PostgreSQL",
            vec!["*.sql", "migrations/"],
            vec!["postgresql"],
        );

        // Web development
        self.add_tool(
            "Web Dev",
            vec!["index.html", "webpack.config.js", "vite.config.js"],
            vec!["node"],
        );

        // Data science
        self.add_tool("Jupyter", vec!["*.ipynb"], vec!["python"]);
    }

    /// Add a tool pattern
    fn add_tool(&mut self, name: &str, files: Vec<&str>, packages: Vec<&str>) {
        for file in &files {
            self.tool_patterns.insert(
                file.to_string(),
                DetectedTool {
                    name: name.to_string(),
                    patterns: files.iter().map(|s| s.to_string()).collect(),
                    detected_files: Vec::new(),
                    packages: packages.iter().map(|s| s.to_string()).collect(),
                },
            );
        }
    }

    /// Detect tools in a directory
    pub fn detect_tools(&self, dir: &Path) -> Result<Vec<DetectedTool>> {
        let mut detected = HashMap::new();

        // Walk directory (limit depth to avoid scanning too deep)
        for entry in WalkDir::new(dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip hidden directories except for dotfiles in the root
            if path.is_dir() && path != dir {
                if let Some(file_name) = path.file_name() {
                    let name = file_name.to_string_lossy();
                    // Skip hidden directories (except .git which we might want to check)
                    if name.starts_with('.') && name != ".git" {
                        continue;
                    }
                }
            }

            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();

                // Check if file matches any pattern
                for (pattern, tool) in &self.tool_patterns {
                    if self.matches_pattern(&name, pattern) {
                        detected
                            .entry(tool.name.clone())
                            .or_insert_with(|| tool.clone())
                            .detected_files
                            .push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(detected.into_values().collect())
    }

    /// Check if a filename matches a pattern
    fn matches_pattern(&self, filename: &str, pattern: &str) -> bool {
        if pattern.ends_with('/') {
            // Directory pattern
            filename == pattern.trim_end_matches('/')
        } else if pattern.contains('*') {
            // Wildcard pattern
            let pattern = pattern.replace('*', "");
            filename.ends_with(&pattern)
        } else {
            // Exact match
            filename == pattern
        }
    }

    /// Generate package suggestions based on detected tools
    pub fn suggest_packages(&self, dir: &Path) -> Result<Vec<PackageSuggestion<'_>>> {
        let detected_tools = self.detect_tools(dir)?;
        let mut suggestions: Vec<PackageSuggestion> = Vec::new();
        let mut suggested_packages = HashSet::new();

        for tool in detected_tools {
            for pkg_name in &tool.packages {
                // Only suggest if not already suggested and exists in database
                if suggested_packages.contains(pkg_name) {
                    continue;
                }

                if let Some(package) = self.db.get(pkg_name) {
                    suggested_packages.insert(pkg_name.clone());

                    // Calculate relevance based on tool importance
                    let relevance = self.calculate_relevance(&tool, package);

                    suggestions.push(PackageSuggestion {
                        package,
                        reason: format!("Detected {} project files", tool.name),
                        relevance,
                        detected_files: tool.detected_files.clone(),
                    });
                }
            }
        }

        // Sort by relevance (descending), then by popularity
        suggestions.sort_by(|a, b| {
            b.relevance
                .cmp(&a.relevance)
                .then(b.package.popularity.cmp(&a.package.popularity))
        });

        Ok(suggestions)
    }

    /// Calculate relevance score for a suggestion
    fn calculate_relevance(&self, tool: &DetectedTool, package: &PackageInfo) -> u8 {
        // Base score from package popularity
        let mut score = package.popularity;

        // Boost for essential development tools
        let essential_tools = ["git", "node", "python", "docker", "rust", "go"];
        if essential_tools.contains(&package.name.as_str()) {
            score = score.saturating_add(10);
        }

        // Boost based on number of detected files (more files = more likely active project)
        let file_boost = (tool.detected_files.len().min(5) * 2) as u8;
        score = score.saturating_add(file_boost);

        score.min(100) // Cap at 100
    }

    /// Suggest packages based on commonly used tools together
    pub fn suggest_related(&self, installed: &[String]) -> Vec<PackageSuggestion<'_>> {
        let mut suggestions = Vec::new();
        let installed_set: HashSet<_> = installed.iter().collect();

        for pkg_name in installed {
            if let Some(package) = self.db.get(pkg_name) {
                // Suggest related packages that aren't installed
                for related in &package.related {
                    if !installed_set.contains(related) {
                        if let Some(related_pkg) = self.db.get(related) {
                            suggestions.push(PackageSuggestion {
                                package: related_pkg,
                                reason: format!("Works well with {}", package.name),
                                relevance: 70, // Medium-high relevance
                                detected_files: Vec::new(),
                            });
                        }
                    }
                }

                // Suggest alternatives (lower priority)
                for alt in &package.alternatives {
                    if !installed_set.contains(alt) {
                        if let Some(alt_pkg) = self.db.get(alt) {
                            suggestions.push(PackageSuggestion {
                                package: alt_pkg,
                                reason: format!("Alternative to {}", package.name),
                                relevance: 50, // Medium relevance
                                detected_files: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        // Deduplicate and sort
        suggestions.sort_by(|a, b| {
            b.relevance
                .cmp(&a.relevance)
                .then(b.package.popularity.cmp(&a.package.popularity))
        });
        suggestions.dedup_by(|a, b| a.package.name == b.package.name);

        suggestions
    }

    /// Get the package database
    pub fn database(&self) -> &PackageDatabase {
        &self.db
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_nodejs_project() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let engine = SuggestionEngine::new();
        let tools = engine.detect_tools(temp.path()).unwrap();

        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "Node.js"));
    }

    #[test]
    fn test_detect_rust_project() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();

        let engine = SuggestionEngine::new();
        let tools = engine.detect_tools(temp.path()).unwrap();

        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "Rust"));
    }

    #[test]
    fn test_suggest_packages_for_nodejs() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let engine = SuggestionEngine::new();
        let suggestions = engine.suggest_packages(temp.path()).unwrap();

        assert!(!suggestions.is_empty());
        assert!(suggestions
            .iter()
            .any(|s| s.package.name == "node" || s.package.name == "yarn"));
    }

    #[test]
    fn test_detect_multiple_tools() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        fs::write(temp.path().join("Dockerfile"), "FROM node").unwrap();

        let engine = SuggestionEngine::new();
        let tools = engine.detect_tools(temp.path()).unwrap();

        assert!(tools.len() >= 2);
        assert!(tools.iter().any(|t| t.name == "Node.js"));
        assert!(tools.iter().any(|t| t.name == "Docker"));
    }

    #[test]
    fn test_relevance_calculation() {
        let engine = SuggestionEngine::new();
        let tool = DetectedTool {
            name: "Node.js".to_string(),
            patterns: vec!["package.json".to_string()],
            detected_files: vec!["package.json".to_string()],
            packages: vec!["node".to_string()],
        };

        let package = engine.db.get("node").unwrap();
        let relevance = engine.calculate_relevance(&tool, package);

        assert!(relevance > 0);
        assert!(relevance <= 100);
    }

    #[test]
    fn test_suggest_related_packages() {
        let engine = SuggestionEngine::new();
        let installed = vec!["git".to_string()];

        let suggestions = engine.suggest_related(&installed);

        assert!(!suggestions.is_empty());
        // Should suggest related packages like gh, delta, lazygit
        assert!(suggestions
            .iter()
            .any(|s| ["gh", "delta", "lazygit"].contains(&s.package.name.as_str())));
    }

    #[test]
    fn test_pattern_matching() {
        let engine = SuggestionEngine::new();

        assert!(engine.matches_pattern("test.sql", "*.sql"));
        assert!(engine.matches_pattern("Dockerfile", "Dockerfile"));
        assert!(!engine.matches_pattern("test.txt", "*.sql"));
    }

    #[test]
    fn test_deduplication() {
        let engine = SuggestionEngine::new();
        let installed = vec!["node".to_string()];

        let suggestions = engine.suggest_related(&installed);

        // Check no duplicates
        let names: Vec<_> = suggestions.iter().map(|s| &s.package.name).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }
}
