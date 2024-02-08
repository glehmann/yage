mod common;

use crate::common::*;
use predicates::prelude::predicate::str::*;
use predicates::prelude::*;
use std::vec;

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
fn no_args_help() {
    yage!().failure().stdout(is_empty()).stderr(
        contains("A simple tool to manage encrypted secrets in YAML")
            .and(is_match(r"status +Check the encryption status of a YAML file").unwrap()),
    );
}

#[test]
fn help_sub_command() {
    for sub_command in vec![
        "decrypt", "edit", "encrypt", "env", "keygen", "pubkey", "status",
    ] {
        yage!(sub_command, "--help")
            .success()
            .stdout(
                is_match(r"-v, --verbose...\s+Increase logging verbosity")
                    .unwrap()
                    .and(is_match(r"-q, --quiet...\s+Decrease logging verbosity").unwrap()),
            )
            .stderr(is_empty());
    }
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
