use colored::Colorize;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum HeimdallError {
    #[error("Not initialized. Run: heimdal init --repo <url> --profile <name>")]
    NotInitialized,
    #[error("Config error: {0}")]
    Config(String),
    #[error("State error: {0}")]
    State(String),
    #[error("Git error: {0}")]
    Git(String),
    #[error("Symlink error in {path}: {reason}")]
    Symlink { path: String, reason: String },
    #[error("Package error [{manager}]: {reason}")]
    Package { manager: String, reason: String },
    #[error("Profile '{name}' not found")]
    ProfileNotFound { name: String },
    #[error("Hook failed: {command}\n  exit code: {code}")]
    HookFailed { command: String, code: i32 },
    #[error("Import error: {0}")]
    Import(String),
    #[error("Secret error: {0}")]
    Secret(String),
}

pub fn print_error_with_help(err: &HeimdallError) {
    let (causes, solutions) = error_context(err);
    eprintln!("\n{} {}\n", "✗".red().bold(), err.to_string().red().bold());
    if !causes.is_empty() {
        eprintln!("  {}", "This usually happens when:".dimmed());
        for c in &causes {
            eprintln!("    {} {}", "•".dimmed(), c.dimmed());
        }
        eprintln!();
    }
    if !solutions.is_empty() {
        eprintln!("  {}", "What to do:".yellow());
        for (i, s) in solutions.iter().enumerate() {
            eprintln!("    {}. {}", (i + 1).to_string().yellow(), s);
        }
        eprintln!();
    }
    eprintln!("  {} {}", "Docs:".cyan(), "https://github.com/limistah/heimdal".cyan().bold());
}

fn error_context(err: &HeimdallError) -> (Vec<&'static str>, Vec<&'static str>) {
    match err {
        HeimdallError::NotInitialized => (
            vec!["You haven't run 'heimdal init' yet", "The state file was deleted"],
            vec!["heimdal init --repo <git-url> --profile <name>", "heimdal wizard  (interactive setup)"],
        ),
        HeimdallError::Config(_) => (
            vec!["Invalid YAML in heimdal.yaml", "Required field missing"],
            vec!["heimdal validate", "See examples: https://github.com/limistah/heimdal/tree/main/examples"],
        ),
        HeimdallError::Symlink { .. } => (
            vec!["A file already exists at the target path", "Permission denied"],
            vec!["heimdal apply --force  (overwrite)", "heimdal apply --backup  (backup existing)"],
        ),
        HeimdallError::ProfileNotFound { .. } => (
            vec!["Profile name is wrong", "Profile was deleted from heimdal.yaml"],
            vec!["heimdal profile list", "heimdal profile create <name>"],
        ),
        _ => (vec![], vec![]),
    }
}
