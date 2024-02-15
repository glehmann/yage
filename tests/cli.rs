mod common;

use crate::common::*;
use predicates::prelude::predicate::str::*;
use predicates::prelude::*;
// use pretty_assertions::{assert_eq, assert_ne};

#[test]
fn help() {
    yage!("--help")
        .stdout(
            contains("A simple tool to manage encrypted secrets in YAML")
                .and(is_match(r"check +Check the encryption status of a YAML file").unwrap()),
        )
        .stderr(is_empty());
}

#[test]
fn no_args_help() {
    yage_cmd!().assert().failure().stdout(is_empty()).stderr(
        contains("A simple tool to manage encrypted secrets in YAML")
            .and(is_match(r"check +Check the encryption status of a YAML file").unwrap()),
    );
}

#[test]
fn help_sub_command() {
    for sub_command in ["check", "decrypt", "edit", "encrypt", "env", "keygen", "pubkey"] {
        yage!(sub_command, "--help")
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
    yage!("--version").stdout(is_match(r"^yage \d+\.\d+\.\d+\n$").unwrap()).stderr(is_empty());
}

#[test]
fn bad_option() {
    yage_cmd!("--foo")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(is_match(r"^error: .+ '--foo'").unwrap().and(contains("Usage:")));
}

#[test]
fn completion() {
    for shell in &["bash", "fish", "zsh"] {
        yage!("--completion", shell).stdout(is_empty().not()).stderr(is_empty());
    }
}
