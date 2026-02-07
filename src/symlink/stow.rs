use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration parsed from .stowrc file
#[derive(Debug, Clone)]
pub struct StowConfig {
    pub target: Option<PathBuf>,
    pub ignore: Vec<String>,
}

impl StowConfig {
    pub fn new() -> Self {
        Self {
            target: None,
            ignore: Vec::new(),
        }
    }

    /// Parse .stowrc file
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read .stowrc: {}", path.display()))?;

        let mut config = Self::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse directives
            if let Some(target) = line.strip_prefix("--target=") {
                let expanded = shellexpand::full(target)
                    .with_context(|| format!("Failed to expand target path: {}", target))?;
                config.target = Some(PathBuf::from(expanded.as_ref()));
            } else if let Some(pattern) = line.strip_prefix("--ignore=") {
                config.ignore.push(pattern.to_string());
            }
        }

        Ok(config)
    }
}

impl Default for StowConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_stowrc() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "--target=~/test").unwrap();
        writeln!(file, "--ignore=.git").unwrap();
        writeln!(file, "--ignore=.DS_Store").unwrap();
        writeln!(file, "# comment").unwrap();
        writeln!(file).unwrap();

        let config = StowConfig::parse(file.path()).unwrap();
        assert!(config.target.is_some());
        assert_eq!(config.ignore.len(), 2);
        assert!(config.ignore.contains(&".git".to_string()));
    }
}
