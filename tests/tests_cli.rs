mod common;

use assert_cmd::prelude::*;
use predicates::prelude::predicate::str::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn help() {
    yage!("--help")
        .success()
        .stdout(
            contains("A simple tool to manage encrypted secrets in YAML")
                .and(is_match(r"status +Check the encryption status of a YAML file").unwrap()),
        )
        .stderr(is_empty());
}

#[test]
fn version() {
    yage!("--version")
        .success()
        .stdout(is_match(r"^yage \d+\.\d+\.\d+\n$").unwrap())
        .stderr(is_empty());
}

#[test]
fn bad_option() {
    yage!("--foo").failure().stdout(is_empty()).stderr(
        is_match(r"^error: .+ '--foo'")
            .unwrap()
            .and(contains("Usage:")),
    );
}

#[test]
fn completion() {
    for shell in &["bash", "fish", "zsh"] {
        yage!("--completion", shell)
            .success()
            .stdout(is_empty().not())
            .stderr(is_empty());
    }
}
