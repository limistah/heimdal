//! Per-run install report: what was installed, skipped and failed (with
//! error messages) during the most recent `heimdal apply`.
//!
//! The report lives in two places:
//! - `State::last_install_report` in `~/.heimdal/state.json` — the structured
//!   record behind `heimdal packages report`.
//! - `~/.heimdal/logs/install-<timestamp>.log` — a plain-text log with the
//!   *full* output of every failed install, for debugging.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::packages::InstallOutcome;

/// Keep at most this many install logs; older ones are pruned.
const MAX_LOGS: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportEntry {
    /// `PackageManager::field_name()` (e.g. "homebrew", "apt", "npm").
    pub manager: String,
    pub package: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedEntry {
    pub manager: String,
    pub package: String,
    /// Error message reported by the install command.
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallReport {
    pub run_at: DateTime<Utc>,
    #[serde(default)]
    pub installed: Vec<ReportEntry>,
    #[serde(default)]
    pub skipped: Vec<ReportEntry>,
    #[serde(default)]
    pub failed: Vec<FailedEntry>,
    /// Path of the plain-text log written for this run, if any.
    #[serde(default)]
    pub log_path: Option<PathBuf>,
}

fn entries(pairs: &[(String, String)]) -> Vec<ReportEntry> {
    pairs
        .iter()
        .map(|(manager, package)| ReportEntry {
            manager: manager.clone(),
            package: package.clone(),
        })
        .collect()
}

impl InstallReport {
    pub fn from_outcome(outcome: &InstallOutcome, run_at: DateTime<Utc>) -> Self {
        Self {
            run_at,
            installed: entries(&outcome.installed),
            skipped: entries(&outcome.skipped),
            failed: outcome
                .failed
                .iter()
                .map(|f| FailedEntry {
                    manager: f.manager.clone(),
                    package: f.name.clone(),
                    error: f.reason.clone(),
                })
                .collect(),
            log_path: None,
        }
    }

    /// One-line summary, e.g. "3 installed, 12 already installed, 1 failed".
    pub fn summary(&self) -> String {
        format!(
            "{} installed, {} already installed, {} failed",
            self.installed.len(),
            self.skipped.len(),
            self.failed.len()
        )
    }
}

/// `~/.heimdal/logs`
pub fn logs_dir() -> Result<PathBuf> {
    Ok(crate::utils::home_dir()?.join(".heimdal").join("logs"))
}

/// Render the plain-text log for a run. `outcome` supplies the full command
/// output for failures, which the stored report doesn't keep.
pub fn render_log(report: &InstallReport, outcome: &InstallOutcome) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "heimdal install log — {}\n{}\n\n",
        report.run_at.to_rfc3339(),
        report.summary()
    ));

    out.push_str(&format!("== Failed ({}) ==\n", outcome.failed.len()));
    for f in &outcome.failed {
        out.push_str(&format!("\n[{}] {}\n", f.manager, f.name));
        out.push_str(&format!("error: {}\n", f.reason));
        if !f.output.is_empty() {
            out.push_str(&f.output);
            out.push('\n');
        }
    }

    out.push_str(&format!("\n== Installed ({}) ==\n", report.installed.len()));
    for e in &report.installed {
        out.push_str(&format!("[{}] {}\n", e.manager, e.package));
    }

    out.push_str(&format!(
        "\n== Already installed ({}) ==\n",
        report.skipped.len()
    ));
    for e in &report.skipped {
        out.push_str(&format!("[{}] {}\n", e.manager, e.package));
    }
    out
}

/// Write the log for this run into `dir`, prune old logs, and return its path.
pub fn write_log_in(
    dir: &std::path::Path,
    report: &InstallReport,
    outcome: &InstallOutcome,
) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let name = format!("install-{}.log", report.run_at.format("%Y%m%d-%H%M%S"));
    let path = dir.join(name);
    std::fs::write(&path, render_log(report, outcome))?;
    crate::privilege::give_back_to_invoking_user(dir);
    crate::privilege::give_back_to_invoking_user(&path);
    prune_logs(dir, MAX_LOGS);
    Ok(path)
}

fn prune_logs(dir: &std::path::Path, keep: usize) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut logs: Vec<PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("install-") && n.ends_with(".log"))
                .unwrap_or(false)
        })
        .collect();
    if logs.len() <= keep {
        return;
    }
    // Timestamped names sort chronologically.
    logs.sort();
    for old in &logs[..logs.len() - keep] {
        let _ = std::fs::remove_file(old);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::FailedPackage;

    fn sample_outcome() -> InstallOutcome {
        InstallOutcome {
            installed: vec![("homebrew".into(), "ripgrep".into())],
            skipped: vec![("homebrew".into(), "git".into())],
            failed: vec![FailedPackage {
                name: "curl".into(),
                reason: "E: Unable to locate package curl".into(),
                manager: "apt".into(),
                output: "--- stderr ---\nE: Unable to locate package curl".into(),
            }],
        }
    }

    #[test]
    fn test_report_from_outcome() {
        let r = InstallReport::from_outcome(&sample_outcome(), Utc::now());
        assert_eq!(r.installed[0].package, "ripgrep");
        assert_eq!(r.skipped[0].package, "git");
        assert_eq!(r.failed[0].manager, "apt");
        assert_eq!(r.failed[0].error, "E: Unable to locate package curl");
        assert_eq!(r.summary(), "1 installed, 1 already installed, 1 failed");
    }

    #[test]
    fn test_report_json_roundtrip() {
        let r = InstallReport::from_outcome(&sample_outcome(), Utc::now());
        let json = serde_json::to_string(&r).unwrap();
        let back: InstallReport = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn test_log_contains_full_failure_output() {
        let outcome = sample_outcome();
        let r = InstallReport::from_outcome(&outcome, Utc::now());
        let log = render_log(&r, &outcome);
        assert!(log.contains("== Failed (1) =="));
        assert!(log.contains("[apt] curl"));
        assert!(log.contains("--- stderr ---"));
        assert!(log.contains("[homebrew] ripgrep"));
    }

    #[test]
    fn test_write_log_prunes_old_logs() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..(MAX_LOGS + 3) {
            std::fs::write(
                dir.path().join(format!("install-20200101-0000{i:02}.log")),
                "x",
            )
            .unwrap();
        }
        let outcome = sample_outcome();
        let r = InstallReport::from_outcome(&outcome, Utc::now());
        let path = write_log_in(dir.path(), &r, &outcome).unwrap();
        assert!(path.exists());
        let count = std::fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(count, MAX_LOGS);
    }
}
