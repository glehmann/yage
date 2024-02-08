mod common;

use assert_fs::prelude::*;
use predicates::path::eq_file;
use predicates::prelude::predicate::str::*;

use crate::common::*;

#[test]
fn pubkey_to_stdout() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    yage!("pubkey", &key_path)
        .stdout(is_public_key())
        .stdout(eq_file(&pub_path))
        .stderr(is_empty());
}

#[test]
fn pubkey_multiple_to_stdout() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    yage!("pubkey", &key_path1, &key_path2, &key_path3)
        .stdout(contains(format!(
            "{}{}{}",
            read(&pub_path1),
            read(&pub_path2),
            read(&pub_path3),
        )))
        .stderr(is_empty());
}

#[test]
fn pubkey_to_file() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let output_path = tmp.child("private.pub");
    yage!("pubkey", &key_path, "--output", &output_path)
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&output_path), read(&pub_path));
}

#[test]
fn pubkey_from_stdin() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    yage_cmd!("pubkey", "-")
        .write_stdin(read(&key_path))
        .assert()
        .success()
        .stdout(is_public_key())
        .stdout(eq_file(&pub_path))
        .stderr(is_empty());
}

#[test]
fn pubkey_from_option() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    yage!("pubkey", "-k", read(&key_path).trim())
        .stdout(is_public_key())
        .stdout(eq_file(&pub_path))
        .stderr(is_empty());
}

#[test]
fn pubkey_from_multiple_options() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    yage!(
        "pubkey",
        "-k",
        read(&key_path1).trim(),
        "-k",
        read(&key_path2).trim(),
        "-k",
        read(&key_path3).trim()
    )
    .stdout(contains(format!(
        "{}{}{}",
        read(&pub_path1),
        read(&pub_path2),
        read(&pub_path3),
    )))
    .stderr(is_empty());
}

#[test]
fn pubkey_from_options_and_files() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let (key_path4, pub_path4) = create_key(&tmp);
    yage!(
        "pubkey",
        &key_path1,
        "-k",
        read(&key_path2).trim(),
        "-k",
        read(&key_path3).trim(),
        &key_path4
    )
    .stdout(contains(format!(
        "{}{}{}{}",
        read(&pub_path2),
        read(&pub_path3),
        read(&pub_path1),
        read(&pub_path4),
    )))
    .stderr(is_empty());
}
