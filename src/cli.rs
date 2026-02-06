use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "heimdal")]
#[command(author, version, about = "A universal dotfile and system configuration manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interactive setup wizard (recommended for new users)
    Wizard,

    /// Import from existing dotfile managers (Stow, dotbot, etc.)
    Import {
        /// Path to dotfiles directory (defaults to ~/dotfiles)
        #[arg(short, long)]
        path: Option<String>,

        /// Tool to import from (stow, dotbot, auto)
        #[arg(short, long, default_value = "auto")]
        from: String,

        /// Output path for generated heimdal.yaml
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Initialize Heimdal on a new machine
    Init {
        /// Profile name to use (e.g., work-laptop, personal-desktop)
        #[arg(short, long)]
        profile: String,

        /// Git repository URL
        #[arg(short, long)]
        repo: String,

        /// Local path for dotfiles (defaults to ~/.dotfiles)
        #[arg(long)]
        path: Option<String>,
    },

    /// Apply configuration (install packages, create symlinks)
    Apply {
        /// Show what would be done without doing it
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Force overwrite conflicts without prompting
        #[arg(short, long)]
        force: bool,
    },

    /// Sync from remote repository and apply changes
    Sync {
        /// Don't output anything (for cron usage)
        #[arg(short, long)]
        quiet: bool,

        /// Show what would be synced without doing it
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Show current status
    Status {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show local changes compared to repository
    Diff {
        /// Show detailed information (line counts)
        #[arg(short, long)]
        verbose: bool,

        /// Interactive mode to commit or discard changes
        #[arg(short, long)]
        interactive: bool,
    },

    /// Commit changes to dotfiles repository
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Auto-generate commit message based on changes
        #[arg(short, long)]
        auto: bool,

        /// Push to remote after committing
        #[arg(short, long)]
        push: bool,

        /// Specific files to commit (defaults to all changes)
        files: Vec<String>,
    },

    /// Push committed changes to remote
    Push {
        /// Remote name (defaults to 'origin')
        #[arg(short, long)]
        remote: Option<String>,

        /// Branch name (defaults to current branch)
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Pull changes from remote repository
    Pull {
        /// Use rebase instead of merge
        #[arg(short, long)]
        rebase: bool,
    },

    /// List available profiles
    Profiles,

    /// Rollback to a previous version
    Rollback {
        /// Commit hash or tag to rollback to (defaults to previous commit)
        target: Option<String>,
    },

    /// Manage auto-sync
    AutoSync {
        #[command(subcommand)]
        action: AutoSyncAction,
    },

    /// Validate heimdal.yaml configuration
    Validate {
        /// Path to heimdal.yaml (defaults to current directory)
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Show configuration value
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show change history
    History {
        /// Number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Manage packages
    Packages {
        #[command(subcommand)]
        action: PackagesAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PackagesAction {
    /// Add a package to the configuration
    Add {
        /// Package name
        name: String,

        /// Package manager (homebrew, apt, dnf, pacman)
        #[arg(short, long)]
        manager: Option<String>,

        /// Profile to add to (defaults to current profile)
        #[arg(short, long)]
        profile: Option<String>,

        /// Skip installation, just add to config
        #[arg(long)]
        no_install: bool,
    },

    /// Remove a package from the configuration
    Remove {
        /// Package name
        name: String,

        /// Profile to remove from (defaults to current profile)
        #[arg(short, long)]
        profile: Option<String>,

        /// Force removal even if other packages depend on it
        #[arg(short, long)]
        force: bool,

        /// Skip uninstallation, just remove from config
        #[arg(long)]
        no_uninstall: bool,
    },

    /// Search for packages in the database
    Search {
        /// Search query
        query: String,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Show detailed information about a package
    Info {
        /// Package name
        name: String,
    },

    /// List all packages in current profile
    List {
        /// Show only installed packages
        #[arg(short, long)]
        installed: bool,

        /// Profile to list (defaults to current profile)
        #[arg(short, long)]
        profile: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum AutoSyncAction {
    /// Enable auto-sync
    Enable {
        /// Sync interval (e.g., 1h, 30m, 2h)
        #[arg(short, long, default_value = "1h")]
        interval: String,
    },
    /// Disable auto-sync
    Disable,
    /// Show auto-sync status
    Status,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Get configuration value
    Get {
        /// Configuration key (e.g., profile, repo_path)
        key: String,
    },
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Show all configuration
    Show,
}
