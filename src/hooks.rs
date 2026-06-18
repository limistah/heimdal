use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::mpsc;

use anyhow::Result;

use crate::config::HookEntry;
use crate::progress::{HookViewport, StageBar};

pub fn run_hooks(
    hooks: &[HookEntry],
    dry_run: bool,
    stage_bar: &StageBar,
    viewport: &mut HookViewport,
) -> Result<()> {
    for hook in hooks {
        let (cmd, fail_on_error, os_filter): (&str, bool, &[String]) = match hook {
            HookEntry::Simple(s) => (s.as_str(), true, &[]),
            HookEntry::Full {
                command,
                fail_on_error,
                os,
                ..
            } => (command.as_str(), *fail_on_error, os.as_slice()),
        };

        let current_os = crate::utils::os_name();
        if !os_filter.is_empty() && !os_filter.iter().any(|o| o == current_os) {
            crate::utils::verbose(&format!(
                "Hook skipped (OS filter [{}] does not match {}): {}",
                os_filter.join(", "),
                current_os,
                cmd,
            ));
            continue;
        }

        if dry_run {
            crate::utils::info(&format!("Would run hook: {}", cmd));
            continue;
        }

        stage_bar.set_hook_name(cmd);

        let mut child = std::process::Command::new("sh")
            .args(["-c", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (tx, rx) = mpsc::channel::<String>();

        let tx_out = tx.clone();
        let stdout = child.stdout.take().expect("stdout is piped");
        let stdout_thread = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if let Ok(l) = line {
                    let _ = tx_out.send(l);
                }
            }
        });

        let tx_err = tx;
        let stderr = child.stderr.take().expect("stderr is piped");
        let stderr_thread = std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                if let Ok(l) = line {
                    let _ = tx_err.send(l);
                }
            }
        });

        // Drain lines until both reader threads close their senders
        for line in rx {
            viewport.push_line(line);
        }

        stdout_thread.join().ok();
        stderr_thread.join().ok();

        let status = child.wait()?;

        if !status.success() {
            let code = status.code().unwrap_or(-1);
            viewport.flush_above();
            if fail_on_error {
                return Err(crate::error::HeimdallError::HookFailed {
                    command: cmd.to_string(),
                    code,
                }
                .into());
            } else {
                crate::utils::warning(&format!("Hook failed (ignored): {} (exit {})", cmd, code));
            }
        } else {
            viewport.clear();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::ApplyProgress;

    #[test]
    fn run_hooks_empty_list_is_ok() {
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();
        assert!(run_hooks(&[], false, &stage, &mut vp).is_ok());
    }

    #[test]
    fn run_hooks_dry_run_skips_execution() {
        let hook = HookEntry::Simple("exit 1".to_string());
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();
        // Would fail if actually run; dry_run must skip it
        assert!(run_hooks(&[hook], true, &stage, &mut vp).is_ok());
    }

    #[test]
    fn run_hooks_captures_stdout_into_viewport() {
        let hook = HookEntry::Simple("printf 'hello\\nworld\\n'".to_string());
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();
        run_hooks(&[hook], false, &stage, &mut vp).unwrap();
        // After success, viewport is cleared
        assert!(vp.bars.is_empty());
        assert!(vp.buffer.is_empty());
    }

    #[test]
    fn run_hooks_fail_on_error_true_returns_err() {
        let hook = HookEntry::Simple("exit 42".to_string());
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();
        let result = run_hooks(&[hook], false, &stage, &mut vp);
        assert!(result.is_err());
        // After failure, viewport is cleared (flush_above was called)
        assert!(vp.bars.is_empty());
    }

    #[test]
    fn run_hooks_fail_on_error_false_continues() {
        use crate::config::HookEntry;
        let hook = HookEntry::Full {
            command: "exit 1".to_string(),
            fail_on_error: false,
            os: vec![],
            description: None, // NOTE: field is "description", NOT "name"
        };
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();
        // Should NOT return an error
        assert!(run_hooks(&[hook], false, &stage, &mut vp).is_ok());
    }
}
