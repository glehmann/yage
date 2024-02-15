mod common;

use crate::common::*;
use assert_fs::prelude::*;
use predicates::str::{contains, is_empty};
use pretty_assertions::assert_eq;

#[test]
fn decrypt_to_stdout() {
    let (_tmp, key_path, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let output = yage!("decrypt", "--key-file", &key_path, &encrypted_path)
        .stderr(is_empty())
        .get_output()
        .clone();
    assert_eq!(String::from_utf8(output.stdout).unwrap(), read(&yaml_path));
}

#[test]
fn decrypt_to_file() {
    let (tmp, key_path1, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let (key_path2, _) = create_key(&tmp);
    let decrypted_path = tmp.child("file.dec.yaml");
    yage!(
        "decrypt",
        "--key-file",
        &key_path1,
        "-K",
        &key_path2,
        &encrypted_path,
        "--output",
        &decrypted_path
    )
    .stdout(is_empty())
    .stderr(is_empty());
    assert_eq!(read(&decrypted_path), read(&yaml_path));
}

#[test]
fn decrypt_from_stdin() {
    let (tmp, key_path, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let decrypted_path = tmp.child("file.dec.yaml");
    yage_cmd!("decrypt", "--key", read(&key_path).trim(), "-", "--output", &decrypted_path)
        .write_stdin(read(&encrypted_path))
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&decrypted_path), read(&yaml_path));
}

#[test]
fn decrypt_key_stdin() {
    let (tmp, key_path, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let decrypted_path = tmp.child("file.dec.yaml");
    yage_cmd!("decrypt", "--key-file", "-", &encrypted_path, "-o", &decrypted_path)
        .write_stdin(read(&key_path))
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&decrypted_path), read(&yaml_path));
}

#[test]
fn decrypt_in_place() {
    let (_tmp, key_path, _, yaml_path, encrypted_path) = generate_encrypted_file();
    yage!("decrypt", "-k", read(&key_path).trim(), &encrypted_path, "-i")
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&encrypted_path), read(&yaml_path));
}

#[test]
fn decrypt_stdin_in_place() {
    yage_cmd!("decrypt", "--in-place", "-")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: stdin can't be modified in place"));
}

#[test]
fn decrypt_multiple_files_no_in_place() {
    yage_cmd!("decrypt", "foo.yaml", "bar.yaml")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: invalid number of input files"));
}

#[test]
fn decrypt_key_from_env() {
    let (tmp, key_path1, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let (key_path2, _) = create_key(&tmp);
    let decrypted_path = tmp.child("file.dec.yaml");
    yage_cmd!("decrypt", &encrypted_path, "--output", &decrypted_path)
        .env("YAGE_KEY", format!("{},{}", read(&key_path1).trim(), read(&key_path2).trim()))
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&decrypted_path), read(&yaml_path));
}

#[test]
fn decrypt_key_file_from_env() {
    let (tmp, key_path1, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let (key_path2, _) = create_key(&tmp);
    let decrypted_path = tmp.child("file.dec.yaml");
    yage_cmd!("decrypt", &encrypted_path, "--output", &decrypted_path)
        .env("YAGE_KEY_FILE", std::env::join_paths(vec![&key_path1, &key_path2]).unwrap())
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(read(&decrypted_path), read(&yaml_path));
}
