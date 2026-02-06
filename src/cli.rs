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
