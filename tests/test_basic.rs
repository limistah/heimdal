/// Test basic CLI functionality (--version, --help, invalid commands)
///
/// These are sanity tests to ensure the binary runs and provides expected output.
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_version_flag() {
    cargo_bin_cmd!()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("heimdal"))
        .stdout(predicate::str::contains("2.0.1"));
}

#[test]
fn test_help_flag() {
    cargo_bin_cmd!()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn test_invalid_command() {
    cargo_bin_cmd!()
        .arg("nonexistent-command-xyz")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unrecognized subcommand")
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_no_arguments_shows_help() {
    // Heimdal shows help to stderr and exits with error code when no command given
    cargo_bin_cmd!()
        .assert()
        .failure() // Exit code 2
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("Commands:"));
}

#[test]
fn test_verbose_flag() {
    cargo_bin_cmd!()
        .arg("--verbose")
        .arg("--help")
        .assert()
        .success();
}
