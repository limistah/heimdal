use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::config::{DotfileEntry, DotfileMapping, HeimdalConfig, HeimdalMeta, Profile};

#[derive(Debug, Clone, PartialEq)]
pub enum SourceTool {
    Stow,
    Dotbot,
    Chezmoi,
    Yadm,
    Homesick,
}

impl SourceTool {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceTool::Stow => "stow",
            SourceTool::Dotbot => "dotbot",
            SourceTool::Chezmoi => "chezmoi",
            SourceTool::Yadm => "yadm",
            SourceTool::Homesick => "homesick",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "stow" => Some(SourceTool::Stow),
            "dotbot" => Some(SourceTool::Dotbot),
            "chezmoi" => Some(SourceTool::Chezmoi),
            "yadm" => Some(SourceTool::Yadm),
            "homesick" => Some(SourceTool::Homesick),
            _ => None,
        }
    }
}

pub struct ImportResult {
    #[allow(dead_code)]
    pub tool: SourceTool,
    /// (source_relative_path, target_absolute_path)
    pub dotfiles: Vec<(String, String)>,
    pub warnings: Vec<String>,
}

/// Auto-detect the dotfile manager from marker files.
pub fn detect_tool(path: &Path) -> Option<SourceTool> {
    if path.join(".stowrc").exists() || path.join(".stow-local-ignore").exists() {
        return Some(SourceTool::Stow);
    }
    if path.join("install.conf.yaml").exists() || path.join("install.conf.json").exists() {
        return Some(SourceTool::Dotbot);
    }
    if path.join(".chezmoiroot").exists() || path.join(".chezmoi.yaml.tmpl").exists() {
        return Some(SourceTool::Chezmoi);
    }
    if path.join(".yadm").exists() {
        return Some(SourceTool::Yadm);
    }
    if (path.join("home").is_dir() && path.join("home").join(".vim").exists())
        || path.join("Castlefile").exists()
    {
        return Some(SourceTool::Homesick);
    }
    None
}

/// Import dotfile mappings from the given path using the specified tool.
pub fn import_from(path: &Path, tool: Option<SourceTool>) -> Result<ImportResult> {
    let resolved_tool = match tool {
        Some(t) => t,
        None => detect_tool(path).unwrap_or(SourceTool::Stow), // default to stow-style walk
    };

    match resolved_tool {
        SourceTool::Stow => import_stow(path),
        SourceTool::Dotbot => import_dotbot(path),
        SourceTool::Chezmoi => import_chezmoi(path),
        SourceTool::Yadm => import_yadm(path),
        SourceTool::Homesick => import_homesick(path),
    }
}

/// Stow: each top-level file/dir maps to ~/.<name>
fn import_stow(path: &Path) -> Result<ImportResult> {
    let skip = [
        ".git",
        ".stowrc",
        ".stow-local-ignore",
        "README.md",
        "LICENSE",
        ".DS_Store",
    ];
    let mut dotfiles = Vec::new();
    let warnings = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        let src = name.clone();
        let target = format!("~/{}", name);
        dotfiles.push((src, target));
    }

    dotfiles.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ImportResult {
        tool: SourceTool::Stow,
        dotfiles,
        warnings,
    })
}

/// Dotbot: parse install.conf.yaml link section
fn import_dotbot(path: &Path) -> Result<ImportResult> {
    let config_path = path.join("install.conf.yaml");
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "No install.conf.yaml found in '{}'. Is this a dotbot repo?",
            path.display()
        ));
    }

    let content = std::fs::read_to_string(&config_path)?;
    let docs: Vec<serde_yaml_ng::Value> = serde_yaml_ng::from_str(&content).unwrap_or_default();

    let mut dotfiles = Vec::new();
    let warnings = Vec::new();

    for doc in &docs {
        if let Some(link_map) = doc.get("link").and_then(|v| v.as_mapping()) {
            for (target, source) in link_map {
                let target_str = target.as_str().unwrap_or("").to_string();
                let source_str = match source {
                    serde_yaml_ng::Value::String(s) => s.clone(),
                    serde_yaml_ng::Value::Mapping(m) => m
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    _ => continue,
                };
                if !source_str.is_empty() && !target_str.is_empty() {
                    // source is relative to dotfiles dir; target is absolute (~/ path)
                    let target_home = if target_str.starts_with('/') {
                        target_str.replace(&std::env::var("HOME").unwrap_or_default(), "~")
                    } else {
                        target_str
                    };
                    dotfiles.push((source_str, target_home));
                }
            }
        }
    }

    dotfiles.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ImportResult {
        tool: SourceTool::Dotbot,
        dotfiles,
        warnings,
    })
}

/// Chezmoi: list files in source dir, strip chezmoi prefixes
fn import_chezmoi(path: &Path) -> Result<ImportResult> {
    let mut dotfiles = Vec::new();
    let mut warnings = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" || name == ".chezmoiroot" {
            continue;
        }

        // chezmoi prefix transformations:
        // dot_X → .X, private_X → X, readonly_X → X, executable_X → X
        let target_name = name
            .trim_start_matches("private_")
            .trim_start_matches("readonly_")
            .trim_start_matches("executable_");
        let target_name = if let Some(stripped) = target_name.strip_prefix("dot_") {
            format!(".{}", stripped)
        } else {
            target_name.to_string()
        };
        // Strip .tmpl suffix
        let target_name = target_name.trim_end_matches(".tmpl").to_string();

        if target_name.is_empty() {
            warnings.push(format!("Could not determine target for '{}'", name));
            continue;
        }

        dotfiles.push((name, format!("~/{}", target_name)));
    }

    dotfiles.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ImportResult {
        tool: SourceTool::Chezmoi,
        dotfiles,
        warnings,
    })
}

/// Yadm: files in repo root map directly to ~ (yadm uses bare git clone)
fn import_yadm(path: &Path) -> Result<ImportResult> {
    let skip = [".git", ".yadm", "README.md", "LICENSE"];
    let mut dotfiles = Vec::new();
    let warnings = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        dotfiles.push((name.clone(), format!("~/{}", name)));
    }

    dotfiles.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ImportResult {
        tool: SourceTool::Yadm,
        dotfiles,
        warnings,
    })
}

/// Homesick: files live in home/ subdirectory of the castle
fn import_homesick(path: &Path) -> Result<ImportResult> {
    let home_dir = path.join("home");
    if !home_dir.exists() {
        return Err(anyhow::anyhow!(
            "No 'home/' directory found in '{}'. Is this a Homesick castle?",
            path.display()
        ));
    }

    let skip = [".git", "README.md", "LICENSE"];
    let mut dotfiles = Vec::new();
    let warnings = Vec::new();

    for entry in std::fs::read_dir(&home_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        dotfiles.push((format!("home/{}", name), format!("~/{}", name)));
    }

    dotfiles.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ImportResult {
        tool: SourceTool::Homesick,
        dotfiles,
        warnings,
    })
}

/// Generate a heimdal.yaml string from the import result.
pub fn generate_heimdal_yaml(result: &ImportResult, profile_name: &str) -> Result<String> {
    let mut dotfiles = Vec::new();
    for (src, target) in &result.dotfiles {
        if format!("~/{}", src) == *target {
            // Simple case — use shorthand
            dotfiles.push(DotfileEntry::Simple(src.clone()));
        } else {
            dotfiles.push(DotfileEntry::Mapped(DotfileMapping {
                source: src.clone(),
                target: target.clone(),
                when: None,
            }));
        }
    }

    let mut profiles = HashMap::new();
    profiles.insert(
        profile_name.to_string(),
        Profile {
            dotfiles,
            ..Default::default()
        },
    );

    let config = HeimdalConfig {
        heimdal: HeimdalMeta {
            version: "1".to_string(),
            repo: None,
        },
        profiles,
        ignore: vec![],
    };

    Ok(serde_yaml_ng::to_string(&config)?)
}
