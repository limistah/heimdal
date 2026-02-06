use super::{DotfileMapping, DotfileTool, ImportResult, Importer};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Importer for dotbot configurations
pub struct DotbotImporter;

impl Importer for DotbotImporter {
    fn detect(path: &Path) -> bool {
        if !path.exists() || !path.is_dir() {
            return false;
        }

        // Check for common dotbot config files
        path.join("install.conf.yaml").exists()
            || path.join("install.conf.yml").exists()
            || path.join("install").exists() // dotbot install script
    }

    fn import(path: &Path) -> Result<ImportResult> {
        let config = Self::parse_config(path)?;
        let mut dotfiles = Vec::new();
        let mut packages = Vec::new();

        // Process link directives
        for link in &config.links {
            let mappings = Self::process_link(path, link)?;
            dotfiles.extend(mappings);
        }

        // Process shell commands to extract package installations
        for shell_cmd in &config.shell {
            if let Some(pkgs) = Self::extract_packages(&shell_cmd.command) {
                packages.extend(pkgs);
            }
        }

        Ok(ImportResult {
            tool: DotfileTool::Dotbot,
            dotfiles,
            packages,
            stow_compat: false,
        })
    }
}

impl DotbotImporter {
    /// Parse dotbot configuration file
    fn parse_config(path: &Path) -> Result<DotbotConfig> {
        // Try different config file names
        let config_files = vec!["install.conf.yaml", "install.conf.yml"];

        for config_file in config_files {
            let config_path = path.join(config_file);
            if config_path.exists() {
                return Self::parse_yaml(&config_path);
            }
        }

        anyhow::bail!("No dotbot configuration file found")
    }

    /// Parse YAML configuration
    fn parse_yaml(path: &Path) -> Result<DotbotConfig> {
        let content = fs::read_to_string(path).context("Failed to read dotbot config")?;

        // Parse as raw YAML first
        let yaml: serde_yaml::Value =
            serde_yaml::from_str(&content).context("Failed to parse YAML")?;

        let mut config = DotbotConfig::default();

        // Dotbot config is an array of directives
        if let serde_yaml::Value::Sequence(items) = yaml {
            for item in items {
                if let serde_yaml::Value::Mapping(map) = item {
                    // Process each directive type
                    for (key, value) in map {
                        if let serde_yaml::Value::String(directive) = key {
                            match directive.as_str() {
                                "link" => {
                                    config.links.extend(Self::parse_links(value)?);
                                }
                                "shell" => {
                                    config.shell.extend(Self::parse_shell(value)?);
                                }
                                "create" => {
                                    config.create.extend(Self::parse_create(value)?);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(config)
    }

    /// Parse link directives
    fn parse_links(value: serde_yaml::Value) -> Result<Vec<LinkDirective>> {
        let mut links = Vec::new();

        if let serde_yaml::Value::Mapping(map) = value {
            for (dest, source) in map {
                let dest_str = Self::yaml_to_string(dest)?;
                let source_str = Self::yaml_to_string(source)?;

                links.push(LinkDirective {
                    destination: dest_str,
                    source: source_str,
                });
            }
        }

        Ok(links)
    }

    /// Parse shell directives
    fn parse_shell(value: serde_yaml::Value) -> Result<Vec<ShellDirective>> {
        let mut commands = Vec::new();

        if let serde_yaml::Value::Sequence(items) = value {
            for item in items {
                match item {
                    serde_yaml::Value::String(cmd) => {
                        commands.push(ShellDirective {
                            command: cmd,
                            description: None,
                        });
                    }
                    serde_yaml::Value::Sequence(cmd_desc) => {
                        if cmd_desc.len() >= 2 {
                            let cmd = Self::yaml_to_string(cmd_desc[0].clone())?;
                            let desc = Self::yaml_to_string(cmd_desc[1].clone())?;
                            commands.push(ShellDirective {
                                command: cmd,
                                description: Some(desc),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(commands)
    }

    /// Parse create directives
    fn parse_create(value: serde_yaml::Value) -> Result<Vec<String>> {
        let mut dirs = Vec::new();

        if let serde_yaml::Value::Sequence(items) = value {
            for item in items {
                if let serde_yaml::Value::String(dir) = item {
                    dirs.push(dir);
                }
            }
        }

        Ok(dirs)
    }

    /// Convert YAML value to string
    fn yaml_to_string(value: serde_yaml::Value) -> Result<String> {
        match value {
            serde_yaml::Value::String(s) => Ok(s),
            serde_yaml::Value::Number(n) => Ok(n.to_string()),
            serde_yaml::Value::Bool(b) => Ok(b.to_string()),
            _ => Ok(String::new()),
        }
    }

    /// Process a link directive into dotfile mappings
    fn process_link(base_path: &Path, link: &LinkDirective) -> Result<Vec<DotfileMapping>> {
        let mut mappings = Vec::new();

        // Expand home directory
        let destination = shellexpand::tilde(&link.destination);
        let destination_path = PathBuf::from(destination.as_ref());

        // Source is relative to dotfiles directory
        let source_path = base_path.join(&link.source);

        if !source_path.exists() {
            // Warn but don't fail
            eprintln!("Warning: Source file not found: {}", source_path.display());
            return Ok(mappings);
        }

        mappings.push(DotfileMapping {
            source: source_path,
            destination: destination_path,
            category: Self::categorize_file(&link.source),
        });

        Ok(mappings)
    }

    /// Categorize a file based on its name/path
    fn categorize_file(filename: &str) -> Option<String> {
        if filename.contains("bash") || filename.contains("zsh") {
            Some("shell".to_string())
        } else if filename.contains("vim") || filename.contains("nvim") {
            Some("editor".to_string())
        } else if filename.contains("git") {
            Some("git".to_string())
        } else if filename.contains("tmux") {
            Some("tmux".to_string())
        } else {
            None
        }
    }

    /// Extract package names from shell commands
    fn extract_packages(command: &str) -> Option<Vec<String>> {
        let mut packages = Vec::new();

        // Look for common package manager commands
        if command.contains("brew install") {
            // Extract packages after "brew install"
            if let Some(pos) = command.find("brew install") {
                let after = &command[pos + 12..].trim();
                let parts: Vec<&str> = after.split_whitespace().collect();
                for part in parts {
                    if !part.starts_with('-') && !part.is_empty() {
                        packages.push(part.to_string());
                    }
                }
            }
        } else if command.contains("apt install") || command.contains("apt-get install") {
            // Extract packages for apt
            let pattern = if command.contains("apt install") {
                "apt install"
            } else {
                "apt-get install"
            };

            if let Some(pos) = command.find(pattern) {
                let after = &command[pos + pattern.len()..].trim();
                let parts: Vec<&str> = after.split_whitespace().collect();
                for part in parts {
                    if !part.starts_with('-') && part != "sudo" && !part.is_empty() {
                        packages.push(part.to_string());
                    }
                }
            }
        }

        if packages.is_empty() {
            None
        } else {
            Some(packages)
        }
    }
}

/// Dotbot configuration structure
#[derive(Debug, Default, Serialize, Deserialize)]
struct DotbotConfig {
    links: Vec<LinkDirective>,
    shell: Vec<ShellDirective>,
    create: Vec<String>,
}

/// Link directive from dotbot config
#[derive(Debug, Serialize, Deserialize)]
struct LinkDirective {
    destination: String,
    source: String,
}

/// Shell directive from dotbot config
#[derive(Debug, Serialize, Deserialize)]
struct ShellDirective {
    command: String,
    description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_dotbot() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create install.conf.yaml
        fs::write(base.join("install.conf.yaml"), "").unwrap();

        assert!(DotbotImporter::detect(base));
    }

    #[test]
    fn test_parse_simple_config() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        let config = r#"
- link:
    ~/.vimrc: vim/vimrc
    ~/.zshrc: zsh/zshrc

- shell:
    - [git submodule update, Installing submodules]
"#;

        fs::write(base.join("install.conf.yaml"), config).unwrap();

        let parsed = DotbotImporter::parse_config(base).unwrap();
        assert_eq!(parsed.links.len(), 2);
        assert_eq!(parsed.shell.len(), 1);
    }

    #[test]
    fn test_extract_packages() {
        let cmd1 = "brew install vim neovim git";
        let packages = DotbotImporter::extract_packages(cmd1).unwrap();
        assert_eq!(packages.len(), 3);
        assert!(packages.contains(&"vim".to_string()));

        let cmd2 = "sudo apt-get install -y curl wget";
        let packages = DotbotImporter::extract_packages(cmd2).unwrap();
        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&"curl".to_string()));
    }
}
