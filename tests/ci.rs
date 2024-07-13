use assert_cmd::assert::OutputAssertExt;
use std::process::Command;

#[test]
fn hack_feature_powerset_udeps() {
    Command::new("rustup")
        .env("RUSTFLAGS", "-D warnings")
        .args([
            "run",
            "nightly",
            "cargo",
            "hack",
            "--feature-powerset",
            "udeps",
        ])
        .assert()
        .success();
}
