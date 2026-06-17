//! Yarn-style progress UI for the apply command.

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// A package install that failed.
#[derive(Debug, Clone)]
pub struct FailedPackage {
    pub name: String,
    pub reason: String,
}

pub struct ApplyProgress {
    mp: MultiProgress,
    total: u8,
    /// Placeholder bars — one per stage, reused (in-place) when stage becomes active.
    bars: Vec<ProgressBar>,
    enabled: bool,
}

pub struct StageBar {
    bar: ProgressBar,
    label: String,
    n: u8,
    total: u8,
    mp: MultiProgress,
    enabled: bool,
}

impl ApplyProgress {
    /// Create a live progress display showing `total_stages` numbered stages.
    /// All stages are rendered immediately as dim placeholders.
    pub fn new(total_stages: u8) -> Self {
        let mp = MultiProgress::new();
        let mut bars = Vec::with_capacity(total_stages as usize);
        for n in 1..=total_stages {
            let bar = mp.add(ProgressBar::new_spinner());
            bar.set_style(
                ProgressStyle::with_template(&format!("  [{n}/{total_stages}] · {{msg}}")).unwrap(),
            );
            bar.set_message("·");
            bars.push(bar);
        }
        Self {
            mp,
            total: total_stages,
            bars,
            enabled: true,
        }
    }

    /// Create a no-op instance used when verbosity is Quiet.
    pub fn noop() -> Self {
        Self {
            mp: MultiProgress::new(),
            total: 0,
            bars: vec![],
            enabled: false,
        }
    }

    /// Activate stage `n` with the given label. Returns a `StageBar` to finish later.
    /// Reuses the pre-created placeholder bar at index `n-1` (no extra bars added).
    pub fn stage(&self, n: u8, label: &str) -> StageBar {
        if !self.enabled {
            return StageBar {
                bar: ProgressBar::hidden(),
                label: label.to_string(),
                n,
                total: self.total,
                mp: MultiProgress::new(),
                enabled: false,
            };
        }
        // Reuse the placeholder bar at index n-1 — update style in-place.
        let bar = self.bars[(n - 1) as usize].clone();
        bar.set_style(
            ProgressStyle::with_template(&format!(
                "[{n}/{}] {{spinner:.cyan}} {{msg}}",
                self.total
            ))
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        bar.set_message(label.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        StageBar {
            bar,
            label: label.to_string(),
            n,
            total: self.total,
            mp: self.mp.clone(),
            enabled: true,
        }
    }

    /// Suspend the progress display, run `f` (allowing subprocess output
    /// to print cleanly), then redraw.
    pub fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        if self.enabled {
            self.mp.suspend(f)
        } else {
            f()
        }
    }
}

/// Animated progress bar + per-package status line for package installations.
/// Fully implemented in Task 3.
pub struct PackageBar {
    bar: ProgressBar,
    status: ProgressBar,
}

impl PackageBar {
    fn new(mp: &MultiProgress, _total: usize) -> Self {
        let bar = mp.add(ProgressBar::hidden());
        let status = mp.add(ProgressBar::hidden());
        Self { bar, status }
    }

    pub fn record_start(&self, _pkg: &str) {}
    pub fn record_success(&self, _pkg: &str) {}
    pub fn record_failure(&self, _pkg: &str, _reason: &str) {}
    pub fn into_failures(self) -> Vec<FailedPackage> {
        self.bar.finish_and_clear();
        self.status.finish_and_clear();
        vec![]
    }
}

impl StageBar {
    /// Collapse the stage to a green `✓ Label (Xs)` line.
    pub fn finish_success(&self, elapsed: Duration) {
        self.bar.set_style(
            ProgressStyle::with_template(&format!("[{}/{}] ✓ {{msg}}", self.n, self.total))
                .unwrap(),
        );
        self.bar.finish_with_message(format!(
            "{} ({:.1}s)",
            self.label.green(),
            elapsed.as_secs_f64()
        ));
    }

    /// Collapse the stage to a yellow `⚠ Label (Xs) — N failed` line
    /// and print each failure below it.
    pub fn finish_warn(&self, elapsed: Duration, failures: &[FailedPackage]) {
        self.bar.set_style(
            ProgressStyle::with_template(&format!("[{}/{}] ⚠ {{msg}}", self.n, self.total))
                .unwrap(),
        );
        self.bar.finish_with_message(format!(
            "{} ({:.1}s) — {} failed",
            self.label.yellow(),
            elapsed.as_secs_f64(),
            failures.len()
        ));
        for f in failures {
            let _ = self.mp.println(format!(
                "      {} {}: {}",
                "✗".red(),
                f.name.red(),
                f.reason
            ));
        }
    }
    /// Create a `PackageBar` displayed as a child of this stage.
    /// (Implementation added in Task 3.)
    pub fn package_bar(&self, total: usize) -> PackageBar {
        let hidden_mp = MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::hidden());
        PackageBar::new(if self.enabled { &self.mp } else { &hidden_mp }, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_progress_noop_stage_finishes() {
        let p = ApplyProgress::noop();
        let bar = p.stage(1, "Test");
        bar.finish_success(Duration::from_millis(50));
        bar.finish_warn(Duration::from_millis(50), &[]);
    }

    #[test]
    fn test_apply_progress_suspend_returns_value() {
        let p = ApplyProgress::noop();
        let v = p.suspend(|| 42u32);
        assert_eq!(v, 42);
    }

    #[test]
    fn test_stage_finish_success_no_panic() {
        let p = ApplyProgress::new(3);
        let bar = p.stage(1, "Hooks");
        bar.finish_success(Duration::from_millis(100));
    }

    #[test]
    fn test_stage_finish_warn_no_panic() {
        let p = ApplyProgress::new(3);
        let bar = p.stage(2, "Packages");
        let failures = vec![FailedPackage {
            name: "wget".to_string(),
            reason: "not found".to_string(),
        }];
        bar.finish_warn(Duration::from_millis(5000), &failures);
    }
}
