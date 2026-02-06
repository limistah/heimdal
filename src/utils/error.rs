use colored::Colorize;

/// Format an error message with helpful context, causes, and solutions
pub fn format_error_with_help(error: &str, causes: Vec<String>, solutions: Vec<String>) -> String {
    let mut output = String::new();

    // Error message
    output.push_str(&format!("{} {}\n\n", "✗".red().bold(), error.red().bold()));

    // Causes
    if !causes.is_empty() {
        output.push_str(&format!("  {}\n", "This usually happens when:".dimmed()));
        for cause in causes {
            output.push_str(&format!("    {} {}\n", "•".dimmed(), cause.dimmed()));
        }
        output.push('\n');
    }

    // Solutions
    if !solutions.is_empty() {
        output.push_str(&format!("  {}\n", "Possible solutions:".yellow()));
        for (i, solution) in solutions.iter().enumerate() {
            output.push_str(&format!(
                "    {}. {}\n",
                (i + 1).to_string().yellow(),
                solution
            ));
        }
        output.push('\n');
    }

    output.push_str(&format!(
        "  {} {}\n",
        "Need help?".cyan(),
        "Run: heimdal --help".cyan().bold()
    ));

    output
}

/// Format a symlink error with helpful suggestions
pub fn symlink_error(file: &str, dest: &str, existing_type: SymlinkErrorType) -> String {
    let (error_msg, causes, solutions) = match existing_type {
        SymlinkErrorType::FileExists => (
            format!("Cannot create symlink for {}", file),
            vec![
                "The file already exists and is not a symlink".to_string(),
                "Another tool is managing it".to_string(),
                "You manually copied it".to_string(),
            ],
            vec![
                "Back it up: heimdal apply --backup".to_string(),
                format!("See the difference: diff {} ~/dotfiles/{}", dest, file),
                "Force overwrite: heimdal apply --force".to_string(),
            ],
        ),
        SymlinkErrorType::PermissionDenied => (
            format!("Permission denied when creating symlink for {}", file),
            vec![
                "The target directory requires elevated permissions".to_string(),
                "You don't have write access to the target location".to_string(),
            ],
            vec![
                "Run with sudo if needed: sudo heimdal apply".to_string(),
                format!("Check permissions: ls -la {}", dest),
                format!("Change owner: sudo chown $USER {}", dest),
            ],
        ),
        SymlinkErrorType::DirectoryNotFound => (
            format!("Cannot create symlink for {}", file),
            vec![
                "The parent directory doesn't exist".to_string(),
                "The target path is invalid".to_string(),
            ],
            vec![
                "Create parent directories manually".to_string(),
                format!(
                    "Check if directory exists: ls -d {}",
                    std::path::Path::new(dest)
                        .parent()
                        .unwrap_or(std::path::Path::new(dest))
                        .display()
                ),
            ],
        ),
    };

    format_error_with_help(&error_msg, causes, solutions)
}

/// Type of symlink error
pub enum SymlinkErrorType {
    FileExists,
    PermissionDenied,
    DirectoryNotFound,
}

/// Format a package installation error with helpful suggestions
pub fn package_error(package: &str, manager: &str, error_type: PackageErrorType) -> String {
    let (error_msg, causes, solutions) = match error_type {
        PackageErrorType::PackageNotFound => (
            format!("Package '{}' not found in {}", package, manager),
            vec![
                "The package name might be different on this system".to_string(),
                "The package might not be available in your repositories".to_string(),
                "There might be a typo in the package name".to_string(),
            ],
            vec![
                format!("Search for it: {} search {}", manager, package),
                format!(
                    "Check available packages: {} list | grep {}",
                    manager, package
                ),
                "Check Heimdal docs for package name mappings".to_string(),
            ],
        ),
        PackageErrorType::ManagerNotFound => (
            format!("Package manager '{}' not found", manager),
            vec![
                "The package manager is not installed".to_string(),
                "The package manager is not in your PATH".to_string(),
                "You're on a different OS than expected".to_string(),
            ],
            vec![
                format!("Install {}: visit official website", manager),
                format!("Check if installed: which {}", manager),
                "Check your OS: heimdal status --verbose".to_string(),
            ],
        ),
        PackageErrorType::InstallationFailed(ref reason) => (
            format!("Failed to install package '{}' via {}", package, manager),
            vec![
                reason.clone(),
                "Network connection issues".to_string(),
                "Insufficient disk space".to_string(),
            ],
            vec![
                format!("Try manually: {} install {}", manager, package),
                "Check your internet connection".to_string(),
                "Check available disk space: df -h".to_string(),
                format!("View logs: {} log", manager),
            ],
        ),
    };

    format_error_with_help(&error_msg, causes, solutions)
}

/// Type of package error
pub enum PackageErrorType {
    PackageNotFound,
    ManagerNotFound,
    InstallationFailed(String),
}

/// Format a configuration error
pub fn config_error(file: &str, error_type: ConfigErrorType) -> String {
    let (error_msg, causes, solutions) = match error_type {
        ConfigErrorType::FileNotFound => (
            format!("Configuration file not found: {}", file),
            vec![
                "You haven't run 'heimdal init' yet".to_string(),
                "The file was deleted".to_string(),
                "You're in the wrong directory".to_string(),
            ],
            vec![
                "Initialize: heimdal init".to_string(),
                format!("Check if file exists: ls {}", file),
                "Run from your dotfiles directory".to_string(),
            ],
        ),
        ConfigErrorType::ParseError(ref reason) => (
            format!("Failed to parse configuration file: {}", file),
            vec![
                reason.clone(),
                "Invalid YAML syntax".to_string(),
                "Unsupported configuration option".to_string(),
            ],
            vec![
                "Validate YAML: heimdal validate".to_string(),
                format!("Check syntax: cat {} | yaml-lint", file),
                "See example configs: https://github.com/limistah/heimdal/tree/main/examples"
                    .to_string(),
            ],
        ),
        ConfigErrorType::ValidationError(ref field, ref reason) => (
            format!("Invalid configuration in {}: {}", file, field),
            vec![
                reason.clone(),
                "Required field is missing".to_string(),
                "Invalid value for field".to_string(),
            ],
            vec![
                "Check field syntax in documentation".to_string(),
                "See examples: heimdal example --full".to_string(),
                "Validate: heimdal validate".to_string(),
            ],
        ),
    };

    format_error_with_help(&error_msg, causes, solutions)
}

/// Type of configuration error
pub enum ConfigErrorType {
    FileNotFound,
    ParseError(String),
    ValidationError(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error_with_help() {
        let result = format_error_with_help(
            "Test error",
            vec!["Cause 1".to_string(), "Cause 2".to_string()],
            vec!["Solution 1".to_string(), "Solution 2".to_string()],
        );

        assert!(result.contains("Test error"));
        assert!(result.contains("Cause 1"));
        assert!(result.contains("Solution 1"));
    }

    #[test]
    fn test_symlink_error() {
        let result = symlink_error(".vimrc", "/home/user/.vimrc", SymlinkErrorType::FileExists);

        assert!(result.contains(".vimrc"));
        assert!(result.contains("already exists"));
    }

    #[test]
    fn test_package_error() {
        let result = package_error("vim", "apt", PackageErrorType::PackageNotFound);

        assert!(result.contains("vim"));
        assert!(result.contains("apt"));
    }
}
