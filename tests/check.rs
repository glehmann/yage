mod common;

use crate::common::*;
use assert_fs::prelude::*;
use predicates::prelude::predicate::str::*;
// use pretty_assertions::{assert_eq, assert_ne};
use std::{fs::OpenOptions, io::Write};

#[test]
fn check_clear() {
    let tmp = temp_dir();
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    yage_cmd!("check", &yaml_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(": not encrypted"));
}

#[test]
fn check_encrypted() {
    let tmp = temp_dir();
    let (_, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!(
        "encrypt",
        "-R",
        &pub_path,
        &yaml_path,
        "-o",
        &encrypted_path
    )
    .stdout(is_empty())
    .stderr(is_empty());
    yage!("check", &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());
}

#[test]
fn check_mixed() {
    let tmp = temp_dir();
    let (_, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!(
        "encrypt",
        "-R",
        &pub_path,
        &yaml_path,
        "-o",
        &encrypted_path
    )
    .stdout(is_empty())
    .stderr(is_empty());
    yage!("check", &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());
    // append some data to the encrypted file
    {
        OpenOptions::new()
            .append(true)
            .open(&encrypted_path)
            .unwrap()
            .write_all(b"auie: tsrn\n")
            .unwrap();
    }
    yage_cmd!("check", &encrypted_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(": partially encrypted"));
}
