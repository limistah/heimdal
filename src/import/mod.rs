pub mod dotbot;
pub mod stow;

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Represents a tool that manages dotfiles
#[derive(Debug, Clone, PartialEq)]
pub enum DotfileTool {
    Stow,
    Dotbot,
    Chezmoi,
    Yadm,
    Manual,
}

impl DotfileTool {
    pub fn name(&self) -> &str {
        match self {
            DotfileTool::Stow => "GNU Stow",
            DotfileTool::Dotbot => "dotbot",
            DotfileTool::Chezmoi => "chezmoi",
            DotfileTool::Yadm => "yadm",
            DotfileTool::Manual => "Manual/Custom",
        }
    }
}

/// Result of importing from another tool
#[derive(Debug)]
pub struct ImportResult {
    pub tool: DotfileTool,
    pub dotfiles: Vec<DotfileMapping>,
    pub packages: Vec<String>,
    pub stow_compat: bool,
}

/// Maps a source file to its destination
#[derive(Debug, Clone)]
pub struct DotfileMapping {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub category: Option<String>,
}

/// Trait for importing from different dotfile tools
pub trait Importer {
    fn detect(path: &Path) -> bool;
    fn import(path: &Path) -> Result<ImportResult>;
}

/// Detect which dotfile tool is being used in a directory
pub fn detect_tool(path: &Path) -> Option<DotfileTool> {
    if !path.exists() || !path.is_dir() {
        return None;
    }

    // Check for Stow structure (subdirectories with dotfiles)
    if stow::StowImporter::detect(path) {
        return Some(DotfileTool::Stow);
    }

    // Check for dotbot config
    if dotbot::DotbotImporter::detect(path) {
        return Some(DotfileTool::Dotbot);
    }

    // Check for chezmoi
    if path.join(".chezmoi").exists() || path.join(".chezmoiignore").exists() {
        return Some(DotfileTool::Chezmoi);
    }

    // Check for yadm
    if path.join(".yadm").exists() {
        return Some(DotfileTool::Yadm);
    }

    // Default to manual
    Some(DotfileTool::Manual)
}

/// Import from the detected tool
pub fn import_from_tool(path: &Path, tool: &DotfileTool) -> Result<ImportResult> {
    match tool {
        DotfileTool::Stow => stow::StowImporter::import(path),
        DotfileTool::Dotbot => dotbot::DotbotImporter::import(path),
        _ => anyhow::bail!("Import from {} not yet implemented", tool.name()),
    }
}
