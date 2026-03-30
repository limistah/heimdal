use assert_cmd::Command;

#[test]
fn test_wizard_help() {
    // wizard doesn't have --help (it's a bare subcommand), but it should respond to --help
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["wizard", "--help"])
        .assert()
        .success();
}

#[test]
fn test_state_subcommands_help() {
    for sub in &[
        "lock-info",
        "unlock",
        "check-drift",
        "check-conflicts",
        "history",
    ] {
        Command::cargo_bin("heimdal")
            .unwrap()
            .args(["state", sub, "--help"])
            .assert()
            .success();
    }
}

#[test]
fn test_autosync_subcommands_help() {
    for sub in &["enable", "disable", "status"] {
        Command::cargo_bin("heimdal")
            .unwrap()
            .args(["auto-sync", sub, "--help"])
            .assert()
            .success();
    }
}
