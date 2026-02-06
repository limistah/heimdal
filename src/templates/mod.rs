pub mod engine;
pub mod variables;

pub use engine::TemplateEngine;
pub use variables::{get_system_variables, merge_variables};

use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::schema::{HeimdallConfig, Profile};

/// Render all templates for a given profile
/// This function merges system, config, and profile variables, then renders
/// all template files specified in the configuration
pub fn render_templates(
    config: &HeimdallConfig,
    profile: &Profile,
    dotfiles_dir: &Path,
    dry_run: bool,
) -> Result<Vec<RenderedTemplate>> {
    let mut rendered = Vec::new();

    // Merge variables with priority: profile > config > system
    let system_vars = get_system_variables();
    let merged_vars = merge_variables(
        system_vars,
        config.templates.variables.clone(),
        profile.templates.variables.clone(),
    );

    // Create template engine with merged variables
    let engine = TemplateEngine::with_variables(merged_vars);

    // Collect all template files (global + profile)
    let mut template_files = config.templates.files.clone();
    template_files.extend(profile.templates.files.clone());

    // Auto-detect .tmpl files in dotfiles directory if no explicit templates
    if template_files.is_empty() {
        template_files = auto_detect_templates(dotfiles_dir)?;
        if !template_files.is_empty() {
            println!(
                "{} Automatically discovered {} template(s) in {}",
                "ℹ".blue(),
                template_files.len(),
                dotfiles_dir.display()
            );
        }
    }

    // Render each template
    for template_file in template_files {
        let src_path = dotfiles_dir.join(&template_file.src);
        let dest_path =
            if template_file.dest.starts_with('/') || template_file.dest.starts_with('~') {
                PathBuf::from(shellexpand::tilde(&template_file.dest).to_string())
            } else {
                dirs::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
                    .join(&template_file.dest)
            };

        if !src_path.exists() {
            eprintln!(
                "{} Template source not found: {}",
                "⚠".yellow(),
                src_path.display()
            );
            continue;
        }

        if dry_run {
            println!(
                "{} Would render: {} -> {}",
                "→".blue(),
                template_file.src,
                dest_path.display()
            );
        } else {
            engine.render_file(&src_path, &dest_path).with_context(|| {
                format!(
                    "Failed to render template: {} -> {}",
                    src_path.display(),
                    dest_path.display()
                )
            })?;
            println!(
                "{} Rendered: {} -> {}",
                "✓".green(),
                template_file.src,
                dest_path.display()
            );
        }

        rendered.push(RenderedTemplate {
            src: src_path,
            dest: dest_path,
        });
    }

    Ok(rendered)
}

/// Auto-detect template files with .tmpl extension in the dotfiles directory
fn auto_detect_templates(dotfiles_dir: &Path) -> Result<Vec<crate::config::schema::TemplateFile>> {
    use crate::config::schema::TemplateFile;
    use std::fs;

    let mut templates = Vec::new();

    // Only look at top level for now (not recursive)
    if let Ok(entries) = fs::read_dir(dotfiles_dir) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.ends_with(".tmpl") {
                    // Remove .tmpl extension for destination
                    let dest = file_name.trim_end_matches(".tmpl").to_string();
                    templates.push(TemplateFile {
                        src: file_name,
                        dest,
                    });
                }
            }
        }
    }

    Ok(templates)
}

/// Result of a rendered template
#[derive(Debug, Clone)]
pub struct RenderedTemplate {
    pub src: PathBuf,
    pub dest: PathBuf,
}
