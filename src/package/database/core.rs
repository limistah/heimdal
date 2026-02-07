use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::{HashMap, HashSet};

use super::loader::{DatabaseLoader, Package};

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

/// Type of match found during search
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchKind {
    /// Exact name match
    Exact,
    /// Name contains the query
    NameContains,
    /// Description contains the query
    DescriptionContains,
    /// Tag contains the query
    TagContains,
    /// Fuzzy match
    Fuzzy,
}

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult<'a> {
    /// Package information
    pub package: &'a PackageInfo,
    /// Relevance score (higher = more relevant)
    pub score: i64,
    /// Type of match
    pub match_kind: MatchKind,
    /// Whether the package is installed
    pub installed: bool,
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

    /// Parse a category from a string (case-insensitive)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "essential" | "git" => PackageCategory::Essential,
            "development" | "build" => PackageCategory::Development,
            "terminal" | "terminals" => PackageCategory::Terminal,
            "editor" | "editors" => PackageCategory::Editor,
            "language" | "languages" => PackageCategory::Language,
            "container" | "containers" => PackageCategory::Container,
            "infrastructure" => PackageCategory::Infrastructure,
            "database" | "databases" => PackageCategory::Database,
            "network" => PackageCategory::Network,
            "application" | "applications" => PackageCategory::Application,
            "data-science" | "datascience" => PackageCategory::DataScience,
            "documentation" => PackageCategory::Documentation,
            "shell" => PackageCategory::Terminal,
            _ => PackageCategory::Other,
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

    /// Search packages with fuzzy matching and scoring
    pub fn search_fuzzy(&self, query: &str) -> Vec<SearchResult<'_>> {
        // Return empty for empty queries
        if query.trim().is_empty() {
            return Vec::new();
        }

        let matcher = SkimMatcherV2::default();
        let query_lower = query.to_lowercase();

        let mut results: Vec<SearchResult> = self
            .packages
            .values()
            .filter_map(|pkg| {
                // Try exact substring match first (highest score)
                let name_lower = pkg.name.to_lowercase();
                let desc_lower = pkg.description.to_lowercase();

                let (score, match_kind) = if name_lower == query_lower {
                    // Exact name match: highest priority
                    (10000 + (pkg.popularity as i64 * 10), MatchKind::Exact)
                } else if name_lower.contains(&query_lower) {
                    // Name contains query: very high priority
                    (5000 + (pkg.popularity as i64 * 10), MatchKind::NameContains)
                } else if desc_lower.contains(&query_lower) {
                    // Description contains query: high priority
                    (
                        2500 + (pkg.popularity as i64 * 5),
                        MatchKind::DescriptionContains,
                    )
                } else if pkg
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
                {
                    // Tag contains query: medium-high priority
                    (1500 + (pkg.popularity as i64 * 5), MatchKind::TagContains)
                } else {
                    // Try fuzzy matching
                    let name_score = matcher.fuzzy_match(&pkg.name, query).unwrap_or(0);
                    let desc_score = matcher.fuzzy_match(&pkg.description, query).unwrap_or(0) / 2;
                    let tag_score = pkg
                        .tags
                        .iter()
                        .filter_map(|t| matcher.fuzzy_match(t, query))
                        .max()
                        .unwrap_or(0)
                        / 2;

                    let fuzzy_score = name_score.max(desc_score).max(tag_score);

                    if fuzzy_score > 0 {
                        (fuzzy_score + (pkg.popularity as i64), MatchKind::Fuzzy)
                    } else {
                        return None; // No match
                    }
                };

                Some(SearchResult {
                    package: pkg,
                    score,
                    match_kind,
                    installed: false, // Will be updated by caller
                })
            })
            .collect();

        // Sort by score (descending), then by popularity, then alphabetically
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then(b.package.popularity.cmp(&a.package.popularity))
                .then(a.package.name.cmp(&b.package.name))
        });

        results
    }

    /// Search packages by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&PackageInfo> {
        let tag_lower = tag.to_lowercase();
        let mut results: Vec<&PackageInfo> = self
            .packages
            .values()
            .filter(|pkg| {
                pkg.tags.iter().any(|t| {
                    let t_lower = t.to_lowercase();
                    t_lower == tag_lower || t_lower.contains(&tag_lower)
                })
            })
            .collect();

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

    /// Check if a package is installed on the system
    /// This checks the most common package managers for the current platform
    pub fn is_package_installed(package_name: &str) -> bool {
        use std::process::Command;

        // macOS: Check Homebrew
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("brew").args(["list", "--formula"]).output() {
                if output.status.success() {
                    let installed = String::from_utf8_lossy(&output.stdout);
                    if installed.lines().any(|line| line == package_name) {
                        return true;
                    }
                }
            }

            // Check casks
            if let Ok(output) = Command::new("brew").args(["list", "--cask"]).output() {
                if output.status.success() {
                    let installed = String::from_utf8_lossy(&output.stdout);
                    if installed.lines().any(|line| line == package_name) {
                        return true;
                    }
                }
            }
        }

        // Linux: Check various package managers
        #[cfg(target_os = "linux")]
        {
            // Check dpkg/apt (use dpkg-query for exact match)
            if let Ok(output) = Command::new("dpkg-query")
                .args(["-W", "-f=${Status} ${Package}\n", package_name])
                .output()
            {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout);
                    // Parse each line: "install ok installed <package_name>"
                    for line in result.lines() {
                        let tokens: Vec<&str> = line.split_whitespace().collect();
                        // Check for "install ok installed" status and exact package name match
                        if tokens.len() >= 4
                            && tokens[0] == "install"
                            && tokens[1] == "ok"
                            && tokens[2] == "installed"
                            && tokens.last() == Some(&package_name)
                        {
                            return true;
                        }
                    }
                }
            }

            // Check dnf/rpm
            if let Ok(output) = Command::new("rpm").args(["-q", package_name]).output() {
                if output.status.success() {
                    return true;
                }
            }

            // Check pacman
            if let Ok(output) = Command::new("pacman").args(["-Q", package_name]).output() {
                if output.status.success() {
                    return true;
                }
            }
        }

        false
    }

    /// Get all installed packages as a HashSet for efficient lookups
    /// This is more efficient than calling is_package_installed() in a loop
    pub fn get_installed_packages() -> HashSet<String> {
        use std::process::Command;
        let mut installed = HashSet::new();

        // macOS: Check Homebrew
        #[cfg(target_os = "macos")]
        {
            // Get formulae
            if let Ok(output) = Command::new("brew").args(["list", "--formula"]).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    installed.extend(stdout.lines().map(|s| s.to_string()));
                }
            }

            // Get casks
            if let Ok(output) = Command::new("brew").args(["list", "--cask"]).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    installed.extend(stdout.lines().map(|s| s.to_string()));
                }
            }
        }

        // Linux: Check various package managers
        #[cfg(target_os = "linux")]
        {
            // Check dpkg/apt
            if let Ok(output) = Command::new("dpkg-query")
                .args(["-W", "-f=${Status} ${Package}\n"])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.contains("install ok installed") {
                            if let Some(pkg_name) = line.split_whitespace().last() {
                                installed.insert(pkg_name.to_string());
                            }
                        }
                    }
                }
            }

            // Check rpm
            if let Ok(output) = Command::new("rpm")
                .args(["-qa", "--queryformat", "%{NAME}\n"])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    installed.extend(stdout.lines().map(|s| s.to_string()));
                }
            }

            // Check pacman
            if let Ok(output) = Command::new("pacman").args(["-Q"]).output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if let Some(pkg_name) = line.split_whitespace().next() {
                            installed.insert(pkg_name.to_string());
                        }
                    }
                }
            }
        }

        installed
    }

    /// Populate the database by loading from external source
    fn populate_packages(&mut self) {
        // Load from external database with auto-update (7 days)
        let loader = match DatabaseLoader::new() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Warning: Failed to initialize database loader: {}", e);
                return;
            }
        };

        let compiled_db = match loader.load_with_auto_update(7) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Warning: Failed to load package database: {}", e);
                eprintln!("Some package features may be unavailable.");
                return;
            }
        };

        // Convert compiled packages to PackageInfo
        for compiled_pkg in compiled_db.packages {
            let package_info = Self::convert_compiled_package(&compiled_pkg);
            self.add(package_info);
        }
    }

    /// Convert a Package to PackageInfo
    fn convert_compiled_package(pkg: &Package) -> PackageInfo {
        PackageInfo {
            name: pkg.name.clone(),
            description: pkg.description.clone(),
            category: PackageCategory::from_str(&pkg.category),
            popularity: pkg.popularity,
            alternatives: pkg.alternatives.clone(),
            related: pkg.related.clone(),
            tags: pkg.tags.clone(),
        }
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

    #[test]
    fn test_fuzzy_search_exact_match() {
        let db = PackageDatabase::new();
        let results = db.search_fuzzy("git");

        assert!(!results.is_empty());
        // Exact match should have highest score and be first
        assert_eq!(results[0].package.name, "git");
        assert!(results[0].score > 10000); // Exact match score
    }

    #[test]
    fn test_fuzzy_search_with_typo() {
        let db = PackageDatabase::new();
        // "docer" is a typo of "docker" - fuzzy matcher should find it
        // (Using a simpler typo that fuzzy matcher handles better)
        let results = db.search_fuzzy("docer");

        // Fuzzy matching should find docker
        assert!(
            !results.is_empty(),
            "Expected fuzzy match to find results for 'docer'"
        );
        assert!(
            results.iter().any(|r| r.package.name == "docker"),
            "Expected to find 'docker' in fuzzy search results for 'docer'"
        );
    }

    #[test]
    fn test_fuzzy_search_partial() {
        let db = PackageDatabase::new();
        let results = db.search_fuzzy("doc");

        assert!(!results.is_empty());
        // Should find docker and other packages with "doc"
        assert!(results.iter().any(|r| r.package.name == "docker"));
    }

    #[test]
    fn test_fuzzy_search_by_description() {
        let db = PackageDatabase::new();
        let results = db.search_fuzzy("container");

        assert!(!results.is_empty());
        // Should find docker (has "container" in description)
        assert!(results.iter().any(|r| r.package.name == "docker"));
    }

    #[test]
    fn test_fuzzy_search_scoring_order() {
        let db = PackageDatabase::new();
        let results = db.search_fuzzy("kubernetes");

        assert!(!results.is_empty());
        // Results should be sorted by score
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    #[test]
    fn test_search_by_tag_exact() {
        let db = PackageDatabase::new();
        let results = db.search_by_tag("kubernetes");

        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p.name == "kubectl"));
        assert!(results.iter().any(|p| p.name == "helm"));
        assert!(results.iter().any(|p| p.name == "k9s"));
    }

    #[test]
    fn test_search_by_tag_partial() {
        let db = PackageDatabase::new();
        let results = db.search_by_tag("k8s");

        assert!(!results.is_empty());
        // Should find packages with k8s tag
        assert!(results.iter().any(|p| p.name == "kubectl"));
    }

    #[test]
    fn test_fuzzy_search_empty_query() {
        let db = PackageDatabase::new();
        let results = db.search_fuzzy("");

        // Empty query should return no results
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_no_match() {
        let db = PackageDatabase::new();
        let results = db.search_fuzzy("xyzabc123notfound");

        // Should return empty for completely unmatched query
        assert!(results.is_empty());
    }
}
