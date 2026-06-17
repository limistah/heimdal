use heimdal::utils::{
    atomic_write, ensure_parent_exists, get_verbosity, hostname, set_verbosity, Verbosity,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_atomic_write() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.txt");

    atomic_write(&path, b"hello world").unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_atomic_write_overwrites() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.txt");

    fs::write(&path, "old").unwrap();
    atomic_write(&path, b"new").unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "new");
}

#[test]
fn test_ensure_parent_exists() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("subdir").join("file.txt");

    ensure_parent_exists(&path).unwrap();

    assert!(path.parent().unwrap().exists());
}

#[test]
fn test_ensure_parent_exists_noop_when_exists() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("file.txt");

    // Parent already exists (tmp dir)
    ensure_parent_exists(&path).unwrap();

    assert!(path.parent().unwrap().exists());
}

#[test]
fn test_hostname_returns_string() {
    let host = hostname();
    assert!(!host.is_empty());
}

// NOTE: verbosity tests must run with --test-threads=1 due to shared global state
#[test]
#[serial_test::serial]
fn test_verbosity_set_get_verbose() {
    let prev = get_verbosity();
    set_verbosity(Verbosity::Verbose);
    assert_eq!(get_verbosity(), Verbosity::Verbose);
    set_verbosity(prev);
}

// NOTE: verbosity tests must run with --test-threads=1 due to shared global state
#[test]
fn test_verbosity_set_get_quiet() {
    set_verbosity(Verbosity::Quiet);
    assert_eq!(get_verbosity(), Verbosity::Quiet);
}

// NOTE: verbosity tests must run with --test-threads=1 due to shared global state
#[test]
fn test_verbosity_set_get_normal() {
    set_verbosity(Verbosity::Normal);
    assert_eq!(get_verbosity(), Verbosity::Normal);
}
