mod common;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::path::eq_file;
use predicates::prelude::predicate::str::*;
use std::path::PathBuf;
use std::process::Command;

use crate::common::*;

fn create_key(tmp: &assert_fs::TempDir) -> (PathBuf, PathBuf) {
    let id = uuid::Uuid::new_v4();
    let key_path = tmp.child(format!("{}.key", id.to_string()));
    let public_path = tmp.child(format!("{}.pub", id.to_string()));
    yage!(
        "keygen",
        "--output",
        &key_path.path(),
        "--public",
        &public_path.path()
    )
    .success()
    .stdout(is_empty())
    .stderr(is_match(PUBKEY_INFO_PATTERN).unwrap());
    (key_path.path().into(), public_path.path().into())
}

#[test]
fn pubkey_to_stdout() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let (key_path, pub_path) = create_key(&tmp);
    yage!("pubkey", &key_path)
        .success()
        .stdout(is_match(PUBKEY_PATTERN).unwrap())
        .stdout(eq_file(&pub_path))
        .stderr(is_empty());
}

#[test]
fn pubkey_multiple_to_stdout() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    yage!("pubkey", &key_path1, &key_path2, &key_path3)
        .success()
        .stdout(contains(read(&pub_path1)))
        .stdout(contains(read(&pub_path2)))
        .stdout(contains(read(&pub_path3)))
        .stderr(is_empty());
}

#[test]
fn pubkey_to_file() {
    let tmp = assert_fs::TempDir::new().unwrap();
    let (key_path, pub_path) = create_key(&tmp);
    let tmp = assert_fs::TempDir::new().unwrap();
    let output_path = tmp.child("private.pub");
    yage!("pubkey", &key_path, "--output", &output_path.path())
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&output_path), read(&pub_path));
}
