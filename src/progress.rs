//! Yarn-style progress UI for the apply command.

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
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

#[derive(Debug)]
struct PackageBarState {
    pos: usize,
    total: usize,
    active: Vec<String>,
    completed: Vec<(String, bool)>, // (name, success)
    tick: usize,
    failures: Vec<FailedPackage>,
}

/// Animated progress bar + per-package status line for package installations.
pub struct PackageBar {
    bar: ProgressBar,
    status: ProgressBar,
    state: Arc<Mutex<PackageBarState>>,
    stop: Arc<AtomicBool>,
    tick_thread: Option<std::thread::JoinHandle<()>>,
}

impl std::fmt::Debug for PackageBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackageBar").finish_non_exhaustive()
    }
}

impl PackageBar {
    fn new(mp: &MultiProgress, total: usize) -> Self {
        let bar = mp.add(ProgressBar::new_spinner());
        bar.set_style(ProgressStyle::with_template("      {msg}").unwrap());

        let status = mp.add(ProgressBar::new_spinner());
        status.set_style(ProgressStyle::with_template("      {msg}").unwrap());

        let state = Arc::new(Mutex::new(PackageBarState {
            pos: 0,
            total,
            active: vec![],
            completed: vec![],
            tick: 0,
            failures: vec![],
        }));
        let stop = Arc::new(AtomicBool::new(false));

        // Background thread: advances the tick counter for the leading-edge animation.
        let state_c = Arc::clone(&state);
        let bar_c = bar.clone();
        let status_c = status.clone();
        let stop_c = Arc::clone(&stop);

        let tick_thread = std::thread::spawn(move || {
            while !stop_c.load(Ordering::Relaxed) {
                {
                    let mut s = state_c.lock().unwrap();
                    s.tick = s.tick.wrapping_add(1);
                    Self::render(&bar_c, &status_c, &s);
                }
                std::thread::sleep(Duration::from_millis(80));
            }
        });

        let pb = Self {
            bar,
            status,
            state,
            stop,
            tick_thread: Some(tick_thread),
        };
        {
            let s = pb.state.lock().unwrap();
            Self::render(&pb.bar, &pb.status, &s);
        }
        pb
    }

    fn render(bar: &ProgressBar, status: &ProgressBar, s: &PackageBarState) {
        const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        const WIDTH: usize = 28;

        let filled = (s.pos * WIDTH).checked_div(s.total).unwrap_or(0);
        let tip = if s.pos < s.total {
            SPINNER[s.tick % SPINNER.len()]
        } else {
            "█"
        };
        let empty = WIDTH
            .saturating_sub(filled)
            .saturating_sub(if s.pos < s.total { 1 } else { 0 });

        let bar_line = format!(
            "[{}{}{}]  {}/{}",
            "█".repeat(filled).green(),
            tip.green(),
            "░".repeat(empty).dimmed(),
            s.pos,
            s.total
        );
        bar.set_message(bar_line);

        let mut parts: Vec<String> = Vec::new();
        for (name, ok) in &s.completed {
            if *ok {
                parts.push(format!("{} {}", name.green(), "✓".green()));
            } else {
                parts.push(format!("{} {}", name.red(), "✗".red()));
            }
        }
        for name in &s.active {
            parts.push(name.clone());
        }
        status.set_message(parts.join("  "));
    }

    /// A package has started installing.
    pub fn record_start(&self, pkg: &str) {
        let mut s = self.state.lock().unwrap();
        s.active.push(pkg.to_string());
        Self::render(&self.bar, &self.status, &s);
    }

    /// A package installed successfully.
    pub fn record_success(&self, pkg: &str) {
        let mut s = self.state.lock().unwrap();
        s.active.retain(|p| p != pkg);
        s.completed.push((pkg.to_string(), true));
        s.pos += 1;
        Self::render(&self.bar, &self.status, &s);
    }

    /// A package install failed.
    pub fn record_failure(&self, pkg: &str, reason: &str) {
        let mut s = self.state.lock().unwrap();
        s.active.retain(|p| p != pkg);
        s.completed.push((pkg.to_string(), false));
        s.pos += 1;
        s.failures.push(FailedPackage {
            name: pkg.to_string(),
            reason: reason.to_string(),
        });
        Self::render(&self.bar, &self.status, &s);
    }

    /// Stop animation, clear bars, and return the list of failures.
    pub fn into_failures(mut self) -> Vec<FailedPackage> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.tick_thread.take() {
            let _ = h.join();
        }
        self.bar.finish_and_clear();
        self.status.finish_and_clear();
        // tick_thread is already joined; no other holder of the mutex.
        self.state.lock().unwrap().failures.clone()
    }
}

impl Drop for PackageBar {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.tick_thread.take() {
            let _ = h.join();
        }
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

    #[test]
    fn test_package_bar_records_and_failures() {
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Packages");
        let bar = std::sync::Arc::new(stage.package_bar(3));

        bar.record_start("git");
        bar.record_start("curl");
        bar.record_success("git");
        bar.record_failure("curl", "network error");
        bar.record_start("wget");
        bar.record_success("wget");

        let failures = std::sync::Arc::try_unwrap(bar)
            .expect("arc still borrowed")
            .into_failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].name, "curl");
        assert_eq!(failures[0].reason, "network error");
    }

    #[test]
    fn test_package_bar_no_failures() {
        let p = ApplyProgress::new(1);
        let stage = p.stage(1, "Packages");
        let bar = std::sync::Arc::new(stage.package_bar(2));
        bar.record_start("git");
        bar.record_success("git");
        bar.record_start("curl");
        bar.record_success("curl");
        let failures = std::sync::Arc::try_unwrap(bar)
            .expect("arc still borrowed")
            .into_failures();
        assert!(failures.is_empty());
    }
}
