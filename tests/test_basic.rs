use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn binary_exists() {
    Command::cargo_bin("heimdal").unwrap();
}

#[test]
fn prints_version() {
    Command::cargo_bin("heimdal").unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("heimdal"));
}

#[test]
fn prints_help() {
    Command::cargo_bin("heimdal").unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Universal dotfile manager"))
        .stdout(contains("init"))
        .stdout(contains("apply"))
        .stdout(contains("status"));
}

#[test]
fn all_subcommands_have_help() {
    for cmd in &["init", "apply", "status", "sync", "diff", "commit",
                 "profile", "packages", "template", "secret", "import",
                 "wizard", "validate", "rollback", "state", "auto-sync"] {
        Command::cargo_bin("heimdal").unwrap()
            .args(&[cmd, "--help"])
            .assert()
            .success();
    }
}

#[test]
fn unknown_command_fails_gracefully() {
    Command::cargo_bin("heimdal").unwrap()
        .arg("boguscommand")
        .assert()
        .failure()
        .stderr(contains("error"));
}
