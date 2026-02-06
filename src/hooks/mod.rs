use anyhow::{Context, Result};
use std::process::Command;

use crate::config::HookCommand;
use crate::utils::{error, info, os_name, step, success, warning};

/// Hook execution context - defines when/where a hook is being run
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookContext {
    /// Before applying configuration
    PreApply,
    /// After applying configuration
    PostApply,
    /// Before syncing from remote
    PreSync,
    /// After syncing from remote
    PostSync,
    /// After linking a dotfile
    PostLink,
    /// Before unlinking a dotfile
    PreUnlink,
    /// Before installing a package
    PreInstall,
    /// After installing a package
    PostInstall,
    /// Custom context
    Custom(&'static str),
}

impl HookContext {
    pub fn name(&self) -> &str {
        match self {
            HookContext::PreApply => "pre_apply",
            HookContext::PostApply => "post_apply",
            HookContext::PreSync => "pre_sync",
            HookContext::PostSync => "post_sync",
            HookContext::PostLink => "post_link",
            HookContext::PreUnlink => "pre_unlink",
            HookContext::PreInstall => "pre_install",
            HookContext::PostInstall => "post_install",
            HookContext::Custom(name) => name,
        }
    }
}

/// Result of executing a hook
#[derive(Debug, Clone)]
pub struct HookResult {
    pub command: String,
    pub success: bool,
    pub output: Option<String>,
    pub skipped: bool,
}

/// Execute a list of hooks
pub fn execute_hooks(
    hooks: &[HookCommand],
    dry_run: bool,
    context: HookContext,
) -> Result<Vec<HookResult>> {
    if hooks.is_empty() {
        return Ok(Vec::new());
    }

    info(&format!("Running {} hooks...", context.name()));
    let mut results = Vec::new();

    for hook in hooks {
        let result = execute_hook(hook, dry_run, context)?;
        results.push(result);
    }

    Ok(results)
}

/// Execute a single hook command
pub fn execute_hook(hook: &HookCommand, dry_run: bool, context: HookContext) -> Result<HookResult> {
    match hook {
        HookCommand::Simple(cmd) => execute_simple_hook(cmd, dry_run, context.name(), true),
        HookCommand::Detailed {
            command,
            description,
            os,
            shell: _shell,
            when,
            fail_on_error,
        } => {
            // Check OS filter
            if !os.is_empty() {
                let current_os = os_name();
                if !os.iter().any(|o| o == &current_os) {
                    info(&format!(
                        "Skipping hook (OS mismatch): {}",
                        description.as_deref().unwrap_or(command)
                    ));
                    return Ok(HookResult {
                        command: command.clone(),
                        success: true,
                        output: None,
                        skipped: true,
                    });
                }
            }

            // Check shell filter (TODO: implement shell detection)
            // For now, just run regardless of shell filter

            // Check 'when' condition
            if let Some(condition) = when {
                if !check_condition(condition)? {
                    info(&format!(
                        "Skipping hook (condition not met): {}",
                        description.as_deref().unwrap_or(command)
                    ));
                    return Ok(HookResult {
                        command: command.clone(),
                        success: true,
                        output: None,
                        skipped: true,
                    });
                }
            }

            execute_simple_hook(command, dry_run, context.name(), *fail_on_error)
        }
    }
}

/// Execute a simple hook command
fn execute_simple_hook(
    cmd: &str,
    dry_run: bool,
    context_name: &str,
    fail_on_error: bool,
) -> Result<HookResult> {
    step(&format!("{}: {}", context_name, cmd));

    if dry_run {
        info(&format!("Would run: {}", cmd));
        return Ok(HookResult {
            command: cmd.to_string(),
            success: true,
            output: None,
            skipped: false,
        });
    }

    // Expand tildes in command
    let expanded = shellexpand::tilde(cmd);

    // Run command via shell for proper expansion
    let output = Command::new("sh")
        .arg("-c")
        .arg(expanded.as_ref())
        .output()
        .with_context(|| format!("Failed to execute hook: {}", cmd))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}\n{}", stdout, stderr).trim().to_string();

    if output.status.success() {
        success(&format!("Hook executed: {}", cmd));
        Ok(HookResult {
            command: cmd.to_string(),
            success: true,
            output: if combined.is_empty() {
                None
            } else {
                Some(combined)
            },
            skipped: false,
        })
    } else if fail_on_error {
        error(&format!("Hook failed: {}", cmd));
        if !combined.is_empty() {
            error(&format!("Output: {}", combined));
        }
        anyhow::bail!("Hook execution failed: {}", cmd);
    } else {
        warning(&format!("Hook failed (continuing): {}", cmd));
        Ok(HookResult {
            command: cmd.to_string(),
            success: false,
            output: Some(combined),
            skipped: false,
        })
    }
}

/// Check a condition like "directory_exists:path" or "not_installed"
fn check_condition(condition: &str) -> Result<bool> {
    if condition == "not_installed" {
        // Check if command is already installed (requires context)
        // For now, always return true
        return Ok(true);
    }

    if let Some(path) = condition.strip_prefix("directory_exists:") {
        let expanded = shellexpand::tilde(path);
        return Ok(std::path::Path::new(expanded.as_ref()).is_dir());
    }

    if let Some(path) = condition.strip_prefix("file_exists:") {
        let expanded = shellexpand::tilde(path);
        return Ok(std::path::Path::new(expanded.as_ref()).is_file());
    }

    // Unknown condition, assume true
    warning(&format!("Unknown condition: {}", condition));
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_context_names() {
        assert_eq!(HookContext::PreApply.name(), "pre_apply");
        assert_eq!(HookContext::PostApply.name(), "post_apply");
        assert_eq!(HookContext::PreSync.name(), "pre_sync");
        assert_eq!(HookContext::PostSync.name(), "post_sync");
    }

    #[test]
    fn test_execute_simple_hook_dry_run() {
        let result = execute_simple_hook("echo test", true, "test_context", true).unwrap();
        assert!(result.success);
        assert!(!result.skipped);
    }

    #[test]
    fn test_check_condition_directory_exists() {
        // /tmp should exist on unix systems
        assert!(check_condition("directory_exists:/tmp").unwrap());
        assert!(!check_condition("directory_exists:/nonexistent_xyz_123").unwrap());
    }

    #[test]
    fn test_empty_hooks() {
        let hooks: Vec<HookCommand> = vec![];
        let results = execute_hooks(&hooks, false, HookContext::PreApply).unwrap();
        assert_eq!(results.len(), 0);
    }
}
