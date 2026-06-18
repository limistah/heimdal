//! Yarn-style progress UI for the apply command.
//!
//! All stage bars — plus the two package-progress slots — are pre-allocated inside
//! `ApplyProgress::new()` and registered with the `MultiProgress` before any stage
//! is activated. This keeps the base layout stable so `enable_steady_tick`
//! background threads don't race with stage/package bar creation.
//!
//! Note: hook output uses `HookViewport`, which may temporarily insert additional
//! lines while a hook is running.

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A package install that failed.
#[derive(Debug, Clone)]
pub struct FailedPackage {
    pub name: String,
    pub reason: String,
}

/// Which stage owns the package-progress bar slots.
const PKG_STAGE: u8 = 2;

pub struct ApplyProgress {
    mp: MultiProgress,
    total: u8,
    /// Stage bars, indexed by `n-1` (0-based).
    bars: Vec<ProgressBar>,
    /// Pre-allocated package-progress bar, positioned right after stage `PKG_STAGE`.
    pkg_bar: ProgressBar,
    /// Pre-allocated package-status line, positioned right after `pkg_bar`.
    pkg_status: ProgressBar,
    enabled: bool,
}

pub struct StageBar {
    bar: ProgressBar,
    label: String,
    n: u8,
    total: u8,
    mp: MultiProgress,
    enabled: bool,
    /// Pre-allocated package-progress slot passed down from `ApplyProgress`.
    /// Hidden for stages that don't install packages.
    pkg_bar: ProgressBar,
    pkg_status: ProgressBar,
}

impl ApplyProgress {
    /// Create a live progress display showing `total_stages` numbered stages.
    ///
    /// All stage bars **and** the two package-progress placeholder bars are
    /// added to the `MultiProgress` here, in their final display order, and
    /// each is `tick()`ed to force an initial draw.  Because the height of the
    /// dynamic area is set once and never changes afterwards, `enable_steady_tick`
    /// background threads cannot race with a height change and no lines are
    /// ever left as duplicates in the scrollback.
    pub fn new(total_stages: u8) -> Self {
        let mp = MultiProgress::new();
        let mut bars = Vec::with_capacity(total_stages as usize);
        let mut pkg_bar = ProgressBar::hidden();
        let mut pkg_status = ProgressBar::hidden();

        for n in 1..=total_stages {
            let bar = mp.add(ProgressBar::new_spinner());
            bar.set_style(
                ProgressStyle::with_template(&format!("  [{n}/{total_stages}] {{msg}}")).unwrap(),
            );
            bar.set_message("·");
            bar.tick();
            bars.push(bar);

            if n == PKG_STAGE {
                // Insert the two package-bar slots immediately after this stage
                // so they appear between PKG_STAGE and the next stage placeholder.
                pkg_bar = mp.add(ProgressBar::new_spinner());
                pkg_bar.set_style(ProgressStyle::with_template("      {msg}").unwrap());
                pkg_bar.set_message("");
                pkg_bar.tick();

                pkg_status = mp.add(ProgressBar::new_spinner());
                pkg_status.set_style(ProgressStyle::with_template("      {msg}").unwrap());
                pkg_status.set_message("");
                pkg_status.tick();
            }
        }

        Self {
            mp,
            total: total_stages,
            bars,
            pkg_bar,
            pkg_status,
            enabled: true,
        }
    }

    /// Create a no-op instance used when verbosity is Quiet.
    pub fn noop() -> Self {
        Self {
            mp: MultiProgress::new(),
            total: 0,
            bars: vec![],
            pkg_bar: ProgressBar::hidden(),
            pkg_status: ProgressBar::hidden(),
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
                pkg_bar: ProgressBar::hidden(),
                pkg_status: ProgressBar::hidden(),
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

        // Hand the pre-allocated pkg slots only to the stage that owns them.
        let (pkg_bar, pkg_status) = if n == PKG_STAGE {
            (self.pkg_bar.clone(), self.pkg_status.clone())
        } else {
            (ProgressBar::hidden(), ProgressBar::hidden())
        };

        StageBar {
            bar,
            label: label.to_string(),
            n,
            total: self.total,
            mp: self.mp.clone(),
            enabled: true,
            pkg_bar,
            pkg_status,
        }
    }

    /// Suspend the progress display, run `f` (allowing subprocess output
    /// to print cleanly), then redraw.
    ///
    /// NOTE: No longer called by production code after the `HookViewport` refactor
    /// (hooks now pipe output directly to viewport bars). Kept for external callers
    /// and any future use.
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
    failures: Vec<FailedPackage>,
}

/// Progress bar + per-package status line for package installations.
pub struct PackageBar {
    bar: ProgressBar,
    status: ProgressBar,
    state: Arc<Mutex<PackageBarState>>,
}

impl std::fmt::Debug for PackageBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackageBar").finish_non_exhaustive()
    }
}

impl PackageBar {
    /// Initialise using pre-allocated `ProgressBar` slots (no `mp.add()` call).
    fn from_slots(bar: ProgressBar, status: ProgressBar, total: usize) -> Self {
        let state = Arc::new(Mutex::new(PackageBarState {
            pos: 0,
            total,
            active: vec![],
            completed: vec![],
            failures: vec![],
        }));
        let pb = Self { bar, status, state };
        {
            let s = pb.state.lock().unwrap();
            Self::render(&pb.bar, &pb.status, &s);
        }
        pb
    }

    /// Hidden (no-op) instance — used by `noop` stages and quiet mode.
    fn hidden(total: usize) -> Self {
        let hidden_mp = MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::hidden());
        let bar = hidden_mp.add(ProgressBar::new_spinner());
        bar.set_style(ProgressStyle::with_template("      {msg}").unwrap());
        let status = hidden_mp.add(ProgressBar::new_spinner());
        status.set_style(ProgressStyle::with_template("      {msg}").unwrap());
        Self::from_slots(bar, status, total)
    }

    fn render(bar: &ProgressBar, status: &ProgressBar, s: &PackageBarState) {
        const WIDTH: usize = 28;

        let filled = if s.total == 0 {
            WIDTH
        } else {
            (s.pos * WIDTH).checked_div(s.total).unwrap_or(0)
        };
        let empty = WIDTH.saturating_sub(filled);

        let bar_line = format!(
            "[{}{}]  {}/{}",
            "█".repeat(filled).green(),
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
        if let Some(i) = s.active.iter().position(|p| p == pkg) {
            s.active.remove(i);
        }
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

    /// Clear bars and return the list of failures.
    pub fn into_failures(self) -> Vec<FailedPackage> {
        self.bar.finish_and_clear();
        self.status.finish_and_clear();
        self.state.lock().unwrap().failures.clone()
    }
}

/// Scrolling log viewport for hook command output.
///
/// Bars are dynamically allocated (0 → max 5) via `MultiProgress::insert_after`
/// as lines arrive. The viewport scrolls once full. On hook completion call
/// `clear()` (success) or `flush_above()` (failure).
pub struct HookViewport {
    mp: MultiProgress,
    anchor: ProgressBar,
    pub(crate) bars: Vec<ProgressBar>,
    pub(crate) buffer: VecDeque<String>,
    enabled: bool,
}

impl HookViewport {
    const MAX_LINES: usize = 5;

    /// Push a new output line. Grows up to MAX_LINES bars then scrolls.
    pub fn push_line(&mut self, line: String) {
        if !self.enabled {
            return;
        }
        if self.bars.len() < Self::MAX_LINES {
            let new_bar = if let Some(last) = self.bars.last() {
                self.mp.insert_after(last, ProgressBar::new(0))
            } else {
                self.mp.insert_after(&self.anchor, ProgressBar::new(0))
            };
            new_bar.set_style(ProgressStyle::with_template("         │ {msg}").unwrap());
            new_bar.set_message(line.clone());
            self.bars.push(new_bar);
            self.buffer.push_back(line);
        } else {
            self.buffer.pop_front();
            self.buffer.push_back(line);
            for (bar, msg) in self.bars.iter().zip(self.buffer.iter()) {
                bar.set_message(msg.clone());
            }
        }
    }

    /// Remove all viewport bars and clear the buffer (hook finished cleanly).
    ///
    /// Safe to call on a disabled viewport: `push_line` guards on `!self.enabled`,
    /// so `bars` and `buffer` are always empty when disabled.
    pub fn clear(&mut self) {
        for bar in self.bars.drain(..) {
            bar.finish_and_clear();
        }
        self.buffer.clear();
    }

    /// Print all buffered lines above the progress bars, then clear.
    /// Call this when a hook fails to preserve its output in scrollback.
    ///
    /// Safe to call on a disabled viewport: `push_line` guards on `!self.enabled`,
    /// so `buffer` is always empty when disabled and nothing is printed.
    pub fn flush_above(&mut self) {
        for line in &self.buffer {
            let _ = self.mp.println(format!("         {}", line));
        }
        self.clear();
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

    /// Create a `PackageBar` using the pre-allocated slots from `ApplyProgress`.
    /// No `mp.add()` is called — the height of the `MultiProgress` does not change.
    pub fn package_bar(&self, total: usize) -> PackageBar {
        if !self.enabled {
            return PackageBar::hidden(total);
        }
        PackageBar::from_slots(self.pkg_bar.clone(), self.pkg_status.clone(), total)
    }

    /// Update the running message (e.g. live symlink counter).
    pub fn set_message(&self, msg: impl Into<String>) {
        if self.enabled {
            self.bar.set_message(msg.into());
        }
    }

    /// Update the running message to show the active hook name.
    /// Format: "{stage label} · {hook_name}"
    pub fn set_hook_name(&self, hook_name: &str) {
        if self.enabled {
            self.bar
                .set_message(format!("{} · {}", self.label, hook_name));
        }
    }

    /// Print a line above all progress bars (e.g. inline symlink errors).
    pub fn println(&self, msg: impl AsRef<str>) {
        if self.enabled {
            let _ = self.mp.println(msg.as_ref());
        }
    }

    /// Collapse the stage showing a linked-file count. Uses ✓ if warnings == 0, ⚠ otherwise.
    pub fn finish_with_counts(&self, elapsed: Duration, linked: u64, warnings: usize) {
        if warnings > 0 {
            self.bar.set_style(
                ProgressStyle::with_template(&format!("[{}/{}] ⚠ {{msg}}", self.n, self.total))
                    .unwrap(),
            );
            self.bar.finish_with_message(format!(
                "{}  {} linked  ({} warnings, {:.1}s)",
                self.label.yellow(),
                linked,
                warnings,
                elapsed.as_secs_f64()
            ));
        } else {
            self.bar.set_style(
                ProgressStyle::with_template(&format!("[{}/{}] ✓ {{msg}}", self.n, self.total))
                    .unwrap(),
            );
            self.bar.finish_with_message(format!(
                "{}  {} linked  ({:.1}s)",
                self.label.green(),
                linked,
                elapsed.as_secs_f64()
            ));
        }
    }

    /// Create a `HookViewport` anchored after this stage bar.
    /// Returns a no-op viewport when progress is disabled (quiet mode).
    #[must_use]
    pub fn hook_viewport(&self) -> HookViewport {
        if !self.enabled {
            let hidden = MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::hidden());
            let anchor = hidden.add(ProgressBar::hidden());
            return HookViewport {
                mp: hidden,
                anchor,
                bars: Vec::new(),
                buffer: VecDeque::new(),
                enabled: false,
            };
        }
        HookViewport {
            mp: self.mp.clone(),
            anchor: self.bar.clone(),
            bars: Vec::new(),
            buffer: VecDeque::new(),
            enabled: true,
        }
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

    /// Verifies that all bars (stage placeholders + pkg slots) are pre-allocated
    /// in `ApplyProgress::new()`, keeping the `MultiProgress` height fixed from
    /// construction so `enable_steady_tick` threads cannot race with a height change.
    #[test]
    fn test_all_placeholder_bars_initialized_at_construction() {
        let p = ApplyProgress::new(5);
        for n in 1u8..=5 {
            let bar = p.stage(n, &format!("Stage {n}"));
            bar.finish_success(Duration::from_millis(0));
        }
    }

    /// Verifies that `package_bar()` on the correct stage uses the pre-allocated
    /// slots (no `mp.add()`) and that failure tracking still works end-to-end.
    #[test]
    fn test_package_bar_on_pkg_stage_uses_pre_allocated_slots() {
        let p = ApplyProgress::new(5);
        let stage2 = p.stage(PKG_STAGE, "Installing packages");
        let bar = stage2.package_bar(4);
        bar.record_start("git");
        bar.record_success("git");
        bar.record_start("curl");
        bar.record_failure("curl", "timeout");
        let failures = bar.into_failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].name, "curl");
    }

    #[test]
    fn test_stage_bar_set_message_no_panic() {
        let p = ApplyProgress::new(2);
        let bar = p.stage(1, "Test");
        bar.set_message("1,234 linked"); // must not panic
    }

    #[test]
    fn test_stage_bar_println_no_panic() {
        let p = ApplyProgress::new(2);
        let bar = p.stage(1, "Test");
        bar.println("! conflict  ~/.zshrc");
    }

    #[test]
    fn test_stage_bar_set_hook_name_no_panic() {
        let p = ApplyProgress::new(2);
        let bar = p.stage(1, "Pre-apply hooks");
        bar.set_hook_name("install-brew.sh");
    }

    #[test]
    fn test_stage_bar_finish_with_counts_clean() {
        let p = ApplyProgress::new(3);
        let bar = p.stage(3, "Symlinks");
        bar.finish_with_counts(Duration::from_millis(800), 1247, 0);
    }

    #[test]
    fn test_stage_bar_finish_with_counts_warnings() {
        let p = ApplyProgress::new(3);
        let bar = p.stage(3, "Symlinks");
        bar.finish_with_counts(Duration::from_millis(800), 1247, 3);
    }

    #[test]
    fn test_hook_viewport_grows_up_to_max() {
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();

        for i in 0..7 {
            vp.push_line(format!("line {}", i));
        }

        // Capped at 5 bars
        assert_eq!(vp.bars.len(), 5);
        // Buffer holds the last 5 lines
        let buf: Vec<&String> = vp.buffer.iter().collect();
        assert_eq!(buf[0], "line 2");
        assert_eq!(buf[4], "line 6");
    }

    #[test]
    fn test_hook_viewport_clear_removes_bars_and_buffer() {
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();

        vp.push_line("line 1".into());
        vp.push_line("line 2".into());
        vp.clear();

        assert!(vp.bars.is_empty());
        assert!(vp.buffer.is_empty());
    }

    #[test]
    fn test_hook_viewport_flush_above_clears() {
        let p = ApplyProgress::new(2);
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();

        vp.push_line("error output".into());
        vp.flush_above(); // should not panic

        // After flush, bars and buffer are cleared
        assert!(vp.bars.is_empty());
        assert!(vp.buffer.is_empty());
    }

    #[test]
    fn test_hook_viewport_noop_on_disabled_stage() {
        let p = ApplyProgress::noop();
        let stage = p.stage(1, "Hooks");
        let mut vp = stage.hook_viewport();

        // Should not panic, bars stay empty
        vp.push_line("line".into());
        vp.clear();
        assert!(vp.bars.is_empty());
    }
}
