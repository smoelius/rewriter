use assert_cmd::assert::OutputAssertExt;
use regex::Regex;
use std::{env::remove_var, fs::read_to_string, path::Path, process::Command};
use tempfile::tempdir;

#[ctor::ctor]
fn initialize() {
    unsafe {
        remove_var("CARGO_TERM_COLOR");
    }
}

#[test]
fn dylint() {
    Command::new("cargo")
        .args(["dylint", "--all", "--", "--all-features", "--all-targets"])
        .env("DYLINT_RUSTFLAGS", "--deny warnings")
        .assert()
        .success();
}

#[test]
fn hack_feature_powerset_clippy() {
    hack_feature_powerset("clippy");
}

#[test]
fn hack_feature_powerset_udeps() {
    hack_feature_powerset("udeps");
}

fn hack_feature_powerset(subcommand: &str) {
    Command::new("rustup")
        // smoelius: Remove `CARGO` environment variable to work around:
        // https://github.com/rust-lang/rust/pull/131729
        .env_remove("CARGO")
        .env("RUSTFLAGS", "-D warnings")
        .args([
            "run",
            "nightly",
            "cargo",
            "hack",
            "--feature-powerset",
            subcommand,
        ])
        .assert()
        .success();
}

#[test]
fn markdown_link_check() {
    let tempdir = tempdir().unwrap();

    // smoelius: Pin `markdown-link-check` to version 3.11 until the following issue is resolved:
    // https://github.com/tcort/markdown-link-check/issues/304
    Command::new("npm")
        .args(["install", "markdown-link-check@3.11"])
        .current_dir(&tempdir)
        .assert()
        .success();

    // smoelius: https://github.com/rust-lang/crates.io/issues/788
    let config = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/markdown_link_check.json");

    let readme_md = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");

    Command::new("npx")
        .args([
            "markdown-link-check",
            "--config",
            &config.to_string_lossy(),
            &readme_md.to_string_lossy(),
        ])
        .current_dir(&tempdir)
        .assert()
        .success();
}

#[test]
fn readme_reference_links_are_sorted() {
    let re = Regex::new(r"^\[[^\]]*\]:").unwrap();
    let readme = read_to_string("README.md").unwrap();
    let links = readme
        .lines()
        .filter(|line| re.is_match(line))
        .collect::<Vec<_>>();
    let mut links_sorted = links.clone();
    links_sorted.sort_unstable();
    assert_eq!(links_sorted, links);
}

#[test]
fn readme_reference_links_are_used() {
    let re = Regex::new(r"(?m)^(\[[^\]]*\]):").unwrap();
    let readme = read_to_string("README.md").unwrap();
    for captures in re.captures_iter(&readme) {
        assert_eq!(2, captures.len());
        let m = captures.get(1).unwrap();
        assert!(
            readme[..m.start()].contains(m.as_str()),
            "{} is unused",
            m.as_str()
        );
    }
}
