mod common;

use assert_fs::prelude::*;
use predicates::prelude::predicate::str::*;

use crate::common::*;

#[test]
fn pubkey_to_file() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let output_path = tmp.child("private.pub");
    yage!("pubkey", &key_path, "--output", &output_path)
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&output_path), read(&pub_path));
    // again with the short option
    let output_path = tmp.child("private2.pub");
    yage!("pubkey", &key_path, "-o", &output_path)
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&output_path), read(&pub_path));
}

#[test]
fn pubkey_empty() {
    yage!("pubkey").stdout(is_empty()).stderr(is_empty());
}

#[test]
fn pubkey_from_env() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    yage_cmd!("pubkey")
        .env(
            "YAGE_KEY",
            format!("{},{}", read(&key_path1).trim(), read(&key_path2).trim()),
        )
        .assert()
        .success()
        .stdout(contains(format!(
            "{}{}",
            read(&pub_path1),
            read(&pub_path2)
        )))
        .stderr(is_empty());
}

#[test]
fn pubkey_from_options_files_and_env() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let (key_path4, pub_path4) = create_key(&tmp);
    let (key_path5, _) = create_key(&tmp);
    let (key_path6, _) = create_key(&tmp);
    let (key_path7, pub_path7) = create_key(&tmp);
    let (key_path8, pub_path8) = create_key(&tmp);
    yage_cmd!(
        "pubkey",
        &key_path1,
        "--key",
        read(&key_path2).trim(),
        "-k",
        read(&key_path3).trim(),
        "-",
        &key_path4
    )
    .env(
        "YAGE_KEY",
        format!("{}{}", read(&key_path5).trim(), read(&key_path6).trim()),
    )
    .write_stdin(format!("{}{}", read(&key_path7), read(&key_path8)))
    .assert()
    .success()
    .stdout(contains(format!(
        "{}{}{}{}{}{}",
        read(&pub_path2),
        read(&pub_path3),
        // YAGE_KEY env var is overridden by the -k option
        // read(&pub_path5),
        // read(&pub_path6),
        read(&pub_path1),
        read(&pub_path7),
        read(&pub_path8),
        read(&pub_path4),
    )))
    .stderr(is_empty());
}
