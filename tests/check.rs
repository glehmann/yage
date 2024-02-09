mod common;

use std::{fs::OpenOptions, io::Write};

use assert_fs::prelude::*;
use predicates::prelude::predicate::str::*;

use crate::common::*;

const YAML_CONTENT: &str = "foo: bar
titi:
  toto: 42
array:
- 1
- 2
- 3
empty_map: {}
empty_array: []
empty_string: ''
empty: null
";

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
