mod cli;
mod commands;
mod config;
mod crypto;
mod error;
mod git;
mod hooks;
mod import;
mod packages;
mod profile;
mod secrets;
mod state;
mod symlink;
mod templates;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

fn main() {
    let cli = Cli::parse();
    if cli.no_color {
        colored::control::set_override(false);
    }
    if let Err(e) = run(cli) {
        if let Some(heimdal_err) = e.downcast_ref::<error::HeimdallError>() {
            error::print_error_with_help(heimdal_err);
        } else {
            eprintln!("{} {}", "✗".red().bold(), e);
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(args) => commands::init::run(args),
        Commands::Apply(args) => commands::apply::run(args),
        Commands::Status(args) => commands::status::run(args),
        Commands::Sync(args) => commands::sync::run(args),
        Commands::Diff(args) => commands::diff::run(args),
        Commands::Commit(args) => commands::commit::run(args),
        Commands::Profile { action } => commands::profile::run(action),
        Commands::Packages { action } => commands::packages::run(action),
        Commands::Template { action } => commands::template::run(action),
        Commands::Secret { action } => commands::secret::run(action),
        Commands::Import(args) => commands::import::run(args),
        Commands::Wizard => commands::wizard::run(),
        Commands::Validate(args) => commands::validate::run(args),
        Commands::Rollback(args) => commands::rollback::run(args),
        Commands::State { action } => commands::state::run(action),
        Commands::AutoSync { action } => commands::autosync::run(action),
    }
}
