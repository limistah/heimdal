// Throwaway demo that drives the same progress.rs API apply.rs uses, with
// synthetic timings, so the real terminal rendering can be observed/captured
// without running a real `heimdal apply` (packages, hooks, symlinks).
use heimdal::progress::ApplyProgress;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    let progress = ApplyProgress::new(5);

    // Stage 1: hooks with a scrolling viewport
    let t = Instant::now();
    let stage1 = progress.stage(1, "Pre-apply hooks");
    let mut vp = stage1.hook_viewport();
    stage1.set_hook_name("install-brew.sh");
    for i in 0..8 {
        vp.push_line(format!("brew: installing dependency {i}"));
        sleep(Duration::from_millis(150));
    }
    vp.clear();
    stage1.finish_success(t.elapsed());

    // Stage 2: packages with active/completed churn
    let t = Instant::now();
    let stage2 = progress.stage(2, "Installing packages");
    let pkgs = [
        "neovim", "tmux", "sesh", "fd", "ripgrep", "tree", "zoxide", "lazygit", "gh", "stow", "go",
        "rust", "zig",
    ];
    let bar = stage2.package_bar(pkgs.len());
    for chunk in pkgs.chunks(3) {
        for p in chunk {
            bar.record_start(p);
            sleep(Duration::from_millis(80));
        }
        sleep(Duration::from_millis(300));
        for p in chunk {
            bar.record_success(p);
            sleep(Duration::from_millis(80));
        }
    }
    let failures = bar.into_failures();
    if failures.is_empty() {
        stage2.finish_success(t.elapsed());
    } else {
        stage2.finish_warn(t.elapsed(), &failures);
    }

    // Stage 3: symlinks with a live counter + inline conflict lines
    let t = Instant::now();
    let stage3 = progress.stage(3, "Symlinks");
    for i in 1..=40u64 {
        if i == 17 {
            stage3.println("         ! conflict  ~/.zshrc — file exists");
        }
        stage3.set_message(format!("{} linked", i));
        sleep(Duration::from_millis(25));
    }
    stage3.finish_with_counts(t.elapsed(), 40, 1);

    // Stage 4: templates
    let t = Instant::now();
    let stage4 = progress.stage(4, "Templates");
    sleep(Duration::from_millis(400));
    stage4.finish_success(t.elapsed());

    // Stage 5: post-apply hooks
    let t = Instant::now();
    let stage5 = progress.stage(5, "Post-apply hooks");
    let mut vp5 = stage5.hook_viewport();
    for i in 0..4 {
        vp5.push_line(format!("post-hook line {i}"));
        sleep(Duration::from_millis(150));
    }
    vp5.clear();
    stage5.finish_success(t.elapsed());
}
