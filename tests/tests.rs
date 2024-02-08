mod common;

use crate::common::*;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
// use lipsum::lipsum;
// use predicates::prelude::predicate::path::*;
use predicates::prelude::predicate::str::*;
// use predicates::prelude::*;
use std::fs::read_to_string;
use std::process::Command;

const KEY_PATTERN: &str = r"^AGE-SECRET-KEY-[0-9A-Z]{59}\s*$";
const PUBKEY_PATTERN: &str = r"^[0-9a-z]{62}\s*$";
const PUBKEY_INFO_PATTERN: &str = r"^Public key: [0-9a-z]{62}\s+$";

#[test]
fn keygen_stdout() {
    yage!("keygen")
        .success()
        .stdout(is_match(KEY_PATTERN).unwrap())
        .stderr(is_match(PUBKEY_INFO_PATTERN).unwrap());
}

#[test]
fn keygen_stdout_quiet() {
    yage!("keygen", "-q")
        .success()
        .stdout(is_match(KEY_PATTERN).unwrap())
        .stderr(is_empty());
}

#[test]
fn keygen_to_key_file() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let key_path = tmp.child("private.key");
    yage!("keygen", "--output", &key_path.path())
        .success()
        .stdout(is_empty())
        .stderr(is_match(PUBKEY_INFO_PATTERN).unwrap());
    read_to_string(key_path.path())
        .unwrap()
        .assert(is_match(KEY_PATTERN).unwrap());
}

#[test]
fn keygen_to_public_file() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let public_path = tmp.child("private.pub");
    yage!("keygen", "--public", &public_path.path())
        .success()
        .stdout(is_match(KEY_PATTERN).unwrap())
        .stderr(is_match(PUBKEY_INFO_PATTERN).unwrap());
    read_to_string(public_path.path())
        .unwrap()
        .assert(is_match(PUBKEY_PATTERN).unwrap());
}
