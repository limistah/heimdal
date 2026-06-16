//! macOS defaults subcommand handlers.

use anyhow::Result;

use crate::cli::DefaultsCmd;
use crate::config::CommandContext;
use crate::defaults::{
    diff_all, export_all, export_domains, import_all, import_domains, print_domain_diff,
    resolve_all, Resolution,
};
use crate::utils::{info, success, warning};

pub fn run(action: DefaultsCmd) -> Result<()> {
    let ctx = CommandContext::load()?;

    let defaults_config = match ctx.config.defaults {
        Some(ref c) if c.enabled => c.clone(),
        Some(_) => {
            info("Defaults sync is disabled in config");
            return Ok(());
        }
        None => {
            warning(
                "No 'defaults' section in heimdal.yaml. Add one to enable macOS defaults sync.",
            );
            return Ok(());
        }
    };

    match action {
        DefaultsCmd::Diff { all } => {
            let diffs = diff_all(&ctx.state.dotfiles_path, &defaults_config)?;

            if diffs.is_empty() {
                success("No differences found");
                return Ok(());
            }

            for diff in &diffs {
                if all || diff.has_conflicts() {
                    print_domain_diff(diff);
                }
            }

            let conflict_count = diffs.iter().filter(|d| d.has_conflicts()).count();
            info(&format!(
                "{} domains with differences, {} with conflicts",
                diffs.len(),
                conflict_count
            ));
        }

        DefaultsCmd::Export { domains, dry_run } => {
            if domains.is_empty() {
                export_all(&ctx.state.dotfiles_path, &defaults_config, dry_run)?;
            } else {
                export_domains(
                    &ctx.state.dotfiles_path,
                    &defaults_config,
                    &domains,
                    dry_run,
                )?;
            }
            success("Export complete");
        }

        DefaultsCmd::Import {
            domains,
            dry_run,
            force,
        } => {
            if force {
                if domains.is_empty() {
                    import_all(&ctx.state.dotfiles_path, &defaults_config, dry_run)?;
                } else {
                    import_domains(
                        &ctx.state.dotfiles_path,
                        &defaults_config,
                        &domains,
                        dry_run,
                    )?;
                }
            } else {
                // Interactive mode — show diffs first
                let diffs = diff_all(&ctx.state.dotfiles_path, &defaults_config)?;
                let filtered: Vec<_> = if domains.is_empty() {
                    diffs
                } else {
                    diffs
                        .into_iter()
                        .filter(|d| domains.contains(&d.domain))
                        .collect()
                };

                let resolutions = resolve_all(filtered);
                let to_import: Vec<_> = resolutions
                    .iter()
                    .filter(|(_, r)| *r == Resolution::UseDotfiles)
                    .map(|(d, _)| d.clone())
                    .collect();

                if !to_import.is_empty() {
                    import_domains(
                        &ctx.state.dotfiles_path,
                        &defaults_config,
                        &to_import,
                        dry_run,
                    )?;
                }
            }
            success("Import complete");
        }

        DefaultsCmd::Sync { dry_run } => {
            // Full sync: diff, resolve, then apply resolutions
            let diffs = diff_all(&ctx.state.dotfiles_path, &defaults_config)?;

            if diffs.is_empty() {
                success("Everything in sync");
                return Ok(());
            }

            let resolutions = resolve_all(diffs);

            let to_export: Vec<_> = resolutions
                .iter()
                .filter(|(_, r)| *r == Resolution::UseLocal)
                .map(|(d, _)| d.clone())
                .collect();

            let to_import: Vec<_> = resolutions
                .iter()
                .filter(|(_, r)| *r == Resolution::UseDotfiles)
                .map(|(d, _)| d.clone())
                .collect();

            if !to_export.is_empty() {
                export_domains(
                    &ctx.state.dotfiles_path,
                    &defaults_config,
                    &to_export,
                    dry_run,
                )?;
            }

            if !to_import.is_empty() {
                import_domains(
                    &ctx.state.dotfiles_path,
                    &defaults_config,
                    &to_import,
                    dry_run,
                )?;
            }

            success("Sync complete");
        }
    }

    Ok(())
}
