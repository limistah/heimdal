use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "heimdal", version, about = "Universal dotfile manager")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(
        short,
        long,
        global = true,
        help = "Enable verbose output",
        conflicts_with = "quiet"
    )]
    pub verbose: bool,
    #[arg(
        short,
        long,
        global = true,
        help = "Suppress all output",
        conflicts_with = "verbose"
    )]
    pub quiet: bool,
    #[arg(long, global = true, help = "Disable color output")]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize heimdal on this machine
    Init(InitArgs),
    /// Apply configuration (create symlinks + install packages)
    Apply(ApplyArgs),
    /// Show current status
    Status(StatusArgs),
    /// Pull from remote and apply
    Sync(SyncArgs),
    /// Show local changes compared to repository
    Diff(DiffArgs),
    /// Commit dotfile changes to git
    Commit(CommitArgs),
    /// Profile management
    Profile {
        #[command(subcommand)]
        action: ProfileCmd,
    },
    /// Package management
    Packages {
        #[command(subcommand)]
        action: PackagesCmd,
    },
    /// Template operations
    Template {
        #[command(subcommand)]
        action: TemplateCmd,
    },
    /// Secret management
    Secret {
        #[command(subcommand)]
        action: SecretCmd,
    },
    /// Import from another dotfile manager
    Import(ImportArgs),
    /// Interactive setup wizard
    Wizard,
    /// Validate heimdal.yaml configuration
    Validate(ValidateArgs),
    /// Rollback to a previous state
    Rollback(RollbackArgs),
    /// State management
    State {
        #[command(subcommand)]
        action: StateCmd,
    },
    /// Background sync management
    AutoSync {
        #[command(subcommand)]
        action: AutoSyncCmd,
    },
}

#[derive(Args)]
pub struct InitArgs {
    #[arg(short, long, help = "Git repository URL for your dotfiles")]
    pub repo: String,
    #[arg(short, long, help = "Profile name (e.g. work, personal)")]
    pub profile: String,
    #[arg(long, help = "Local dotfiles path (default: ~/.dotfiles)")]
    pub path: Option<String>,
    #[arg(long, help = "Skip git clone if directory already exists")]
    pub no_clone: bool,
}

#[derive(Args, Default)]
pub struct ApplyArgs {
    #[arg(short = 'n', long, help = "Preview without making changes")]
    pub dry_run: bool,
    #[arg(short, long, help = "Overwrite existing files")]
    pub force: bool,
    #[arg(long, help = "Backup existing files instead of failing")]
    pub backup: bool,
    #[arg(long, help = "Only create symlinks, skip packages")]
    pub dotfiles_only: bool,
    #[arg(long, help = "Only install packages, skip symlinks")]
    pub packages_only: bool,
}

#[derive(Args)]
pub struct StatusArgs {}

#[derive(Args, Default)]
pub struct SyncArgs {
    #[arg(short = 'n', long, help = "Preview without making changes")]
    pub dry_run: bool,
}

#[derive(Args, Default)]
pub struct DiffArgs {
    #[arg(short, long, help = "Show verbose diff output")]
    pub verbose: bool,
}

#[derive(Args)]
pub struct CommitArgs {
    #[arg(short, long, help = "Commit message")]
    pub message: Option<String>,
    #[arg(short, long, help = "Push after committing")]
    pub push: bool,
    pub files: Vec<String>,
}

#[derive(Args)]
pub struct ImportArgs {
    #[arg(
        short,
        long,
        default_value = "~/.dotfiles",
        help = "Path to existing dotfiles"
    )]
    pub path: String,
    #[arg(
        short,
        long,
        help = "Tool to import from (stow, dotbot, chezmoi, yadm, homesick, auto)"
    )]
    pub from: Option<String>,
    #[arg(short, long, help = "Output path for generated heimdal.yaml")]
    pub output: Option<String>,
    #[arg(long, help = "Preview without writing files")]
    pub preview: bool,
}

#[derive(Args)]
pub struct ValidateArgs {
    #[arg(short, long, help = "Path to heimdal.yaml")]
    pub config: Option<String>,
}

#[derive(Args)]
pub struct RollbackArgs {
    #[arg(help = "Commit hash or tag to rollback to (default: previous commit)")]
    pub target: Option<String>,
    #[arg(short = 'n', long, help = "Preview without making changes")]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum ProfileCmd {
    /// Switch to a different profile
    Switch {
        name: String,
        #[arg(long)]
        no_apply: bool,
    },
    /// List all profiles
    List,
    /// Show profile details
    Show {
        name: Option<String>,
        #[arg(short, long)]
        resolved: bool,
    },
    /// Show current profile
    Current,
    /// Create a new profile
    Create {
        name: String,
        #[arg(long)]
        extends: Option<String>,
    },
    /// Clone an existing profile
    Clone { source: String, dest: String },
    /// Compare two profiles
    Diff {
        profile1: Option<String>,
        profile2: String,
    },
}

#[derive(Subcommand)]
pub enum PackagesCmd {
    /// Add a package
    Add {
        name: String,
        #[arg(short, long)]
        manager: Option<String>,
        #[arg(long)]
        no_install: bool,
    },
    /// Remove a package
    Remove {
        name: String,
        #[arg(long)]
        no_uninstall: bool,
    },
    /// List packages
    List {
        #[arg(short, long)]
        installed: bool,
    },
    /// Search packages
    Search { query: String },
    /// Show package info
    Info { name: String },
}

#[derive(Subcommand)]
pub enum TemplateCmd {
    /// Preview rendered template
    Preview {
        src: String,
        #[arg(short, long)]
        profile: Option<String>,
    },
    /// List all templates
    List,
    /// Show available variables
    Variables {
        #[arg(short, long)]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SecretCmd {
    /// Add or update a secret
    Add {
        name: String,
        #[arg(long)]
        value: Option<String>,
    },
    /// Get a secret value
    Get { name: String },
    /// Remove a secret
    Remove {
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    /// List secret names
    List,
}

#[derive(Subcommand)]
pub enum StateCmd {
    /// Show lock status
    LockInfo,
    /// Force-remove lock
    Unlock {
        #[arg(short, long)]
        force: bool,
    },
    /// Check for file drift
    CheckDrift,
    /// Check for state conflicts
    CheckConflicts,
    /// Show operation history
    History {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum AutoSyncCmd {
    /// Enable background sync
    Enable {
        #[arg(short, long, default_value = "1h")]
        interval: String,
    },
    /// Disable background sync
    Disable,
    /// Show sync status
    Status,
}
