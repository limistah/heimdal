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

        /// Preview what would be imported without actually importing
        #[arg(long)]
        preview: bool,
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

    /// Manage git branches
    Branch {
        #[command(subcommand)]
        action: BranchAction,
    },

    /// Manage git remotes
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    /// List available profiles
    Profiles,

    /// Manage profiles (switch, show, list, etc.)
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

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

    /// Manage templates
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Manage secrets (API keys, tokens, passwords)
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },

    /// Manage state (locks, conflicts, migrations, drift detection)
    State {
        #[command(subcommand)]
        action: StateAction,
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

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Suggest packages based on detected project files
    Suggest {
        /// Directory to analyze (defaults to current directory)
        #[arg(short, long)]
        directory: Option<String>,
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

    /// List available package groups
    ListGroups {
        /// Filter by category (development, devops, productivity, etc.)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Show detailed information about a package group
    ShowGroup {
        /// Group ID (e.g., web-dev, rust-dev, terminal)
        id: String,
    },

    /// Install all packages from a group
    AddGroup {
        /// Group ID to install
        id: String,

        /// Include optional packages
        #[arg(short = 'o', long)]
        include_optional: bool,

        /// Show what would be done without doing it
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip installation, just add to config
        #[arg(long)]
        no_install: bool,
    },

    /// Search for package groups
    SearchGroups {
        /// Search query
        query: String,
    },

    /// Update all packages in the current profile
    UpdateAll {
        /// Show what would be updated without doing it
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Show packages with available updates
    Outdated {
        /// Check all installed packages (not just profile packages)
        #[arg(short, long)]
        all: bool,
    },

    /// Upgrade packages to latest versions
    Upgrade {
        /// Specific package to upgrade (upgrades all profile packages if not specified)
        package: Option<String>,

        /// Upgrade all installed packages (not just profile packages)
        #[arg(short, long)]
        all: bool,

        /// Show what would be upgraded without doing it
        #[arg(short = 'n', long)]
        dry_run: bool,
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

#[derive(Subcommand, Debug)]
pub enum BranchAction {
    /// Show current branch
    Current,

    /// List all branches
    List,

    /// Create and switch to a new branch
    Create {
        /// Branch name
        name: String,
    },

    /// Switch to a branch
    Switch {
        /// Branch name
        name: String,
    },

    /// Show tracking information
    Info,
}

#[derive(Subcommand, Debug)]
pub enum RemoteAction {
    /// List all remotes
    List {
        /// Show URLs
        #[arg(short, long)]
        verbose: bool,
    },

    /// Add a new remote
    Add {
        /// Remote name (e.g., origin, upstream)
        name: String,
        /// Remote URL
        url: String,
    },

    /// Remove a remote
    Remove {
        /// Remote name
        name: String,
    },

    /// Change remote URL
    SetUrl {
        /// Remote name
        name: String,
        /// New URL
        url: String,
    },

    /// Show remote details
    Show {
        /// Remote name
        name: String,
    },

    /// Interactive remote setup
    Setup,
}

#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    /// Switch to a different profile
    Switch {
        /// Profile name to switch to
        name: String,

        /// Don't automatically reapply after switching
        #[arg(long)]
        no_apply: bool,
    },

    /// Show currently active profile
    Current,

    /// Show detailed information about a profile
    Show {
        /// Profile name (defaults to current profile)
        name: Option<String>,

        /// Show resolved configuration (after inheritance)
        #[arg(short, long)]
        resolved: bool,
    },

    /// List all available profiles
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Compare two profiles
    Diff {
        /// First profile name (defaults to current)
        profile1: Option<String>,

        /// Second profile name
        profile2: String,
    },

    /// List available profile templates
    Templates,

    /// Create a new profile from a template
    Create {
        /// New profile name
        name: String,

        /// Template to use
        #[arg(short, long)]
        template: String,
    },

    /// Clone an existing profile
    Clone {
        /// Source profile name
        source: String,

        /// New profile name
        target: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TemplateAction {
    /// Preview how a template will be rendered
    Preview {
        /// Template file path
        file: String,

        /// Profile to use for variables (defaults to current)
        #[arg(short, long)]
        profile: Option<String>,
    },

    /// List all template files and variables
    List {
        /// Show variable values
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show all available variables
    Variables {
        /// Profile to show variables for (defaults to current)
        #[arg(short, long)]
        profile: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SecretAction {
    /// Add or update a secret
    Add {
        /// Secret name (e.g., github_token, api_key)
        name: String,

        /// Secret value (will prompt securely if not provided)
        #[arg(long)]
        value: Option<String>,
    },

    /// Get a secret value
    Get {
        /// Secret name
        name: String,
    },

    /// Remove a secret
    Remove {
        /// Secret name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// List all secret names (not values)
    List {
        /// Show creation dates
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum StateAction {
    /// Show current lock status
    LockInfo,

    /// Force remove an active lock
    Unlock {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Check for state conflicts between local and remote
    CheckConflicts,

    /// Resolve detected conflicts
    Resolve {
        /// Resolution strategy (local, remote, merge)
        #[arg(short, long)]
        strategy: String,

        /// Don't prompt for confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Check for file drift (modifications outside heimdal)
    CheckDrift {
        /// Show all files, not just drifted ones
        #[arg(short, long)]
        all: bool,
    },

    /// Show operation history
    History {
        /// Number of operations to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show state version information
    Version,

    /// Migrate from V1 to V2 state format
    Migrate {
        /// Skip backup creation
        #[arg(long)]
        no_backup: bool,

        /// Force migration even if already V2
        #[arg(short, long)]
        force: bool,
    },
}
