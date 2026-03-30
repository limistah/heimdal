use crate::cli::AutoSyncCmd;
use crate::utils::info;
use anyhow::Result;

pub fn run(action: AutoSyncCmd) -> Result<()> {
    match action {
        AutoSyncCmd::Enable { interval } => {
            info(&format!(
                "AutoSync: to enable background sync every {}, add a cron job:",
                interval
            ));
            info("  crontab -e");
            info(&format!(
                "  */60 * * * * heimdal sync  # every hour (adjust for {})",
                interval
            ));
            info("Or use a systemd user timer on Linux.");
            Ok(())
        }
        AutoSyncCmd::Disable => {
            info("AutoSync: remove the cron job or systemd timer you set up.");
            info("  crontab -e  # remove the heimdal sync line");
            Ok(())
        }
        AutoSyncCmd::Status => {
            info("AutoSync: check your cron jobs with 'crontab -l'");
            Ok(())
        }
    }
}
