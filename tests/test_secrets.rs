use assert_cmd::Command;

#[test]
fn test_secret_add_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["secret", "add", "--help"])
        .assert().success();
}

#[test]
fn test_secret_list_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["secret", "list", "--help"])
        .assert().success();
}

#[test]
fn test_secret_get_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["secret", "get", "--help"])
        .assert().success();
}

#[test]
fn test_secret_remove_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["secret", "remove", "--help"])
        .assert().success();
}
