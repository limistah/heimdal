use anyhow::Result;
use crate::config::HookEntry;

pub fn run_hooks(hooks: &[HookEntry], dry_run: bool) -> Result<()> {
    for hook in hooks {
        let (cmd, fail_on_error, os_filter): (&str, bool, &[String]) = match hook {
            HookEntry::Simple(s) => (s.as_str(), true, &[]),
            HookEntry::Full { command, fail_on_error, os, .. } => {
                (command.as_str(), *fail_on_error, os.as_slice())
            }
        };

        if !os_filter.is_empty()
            && !os_filter.iter().any(|o| o == crate::utils::os_name())
        {
            continue;
        }

        if dry_run {
            crate::utils::info(&format!("Would run hook: {}", cmd));
            continue;
        }

        crate::utils::step(&format!("Hook: {}", cmd));
        let status = std::process::Command::new("sh")
            .args(["-c", cmd])
            .status()?;

        if !status.success() && fail_on_error {
            return Err(crate::error::HeimdallError::HookFailed {
                command: cmd.to_string(),
                code: status.code().unwrap_or(-1),
            }.into());
        }
    }
    Ok(())
}
