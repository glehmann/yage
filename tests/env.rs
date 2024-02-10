mod common;

use assert_fs::fixture::PathChild;
use common::*;
use predicates::str::{contains, is_empty};

const YAML_CONTENT: &str = "foo: bar";

#[test]
fn env_key_file_from_args() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
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
    );
    yage!("env", "--key-file", &key_path, &encrypted_path, "env")
        .stdout(contains("foo=bar"))
        .stderr(is_empty());
    // again with short option
    yage!("env", "-K", &key_path, &encrypted_path, "env")
        .stdout(contains("foo=bar"))
        .stderr(is_empty());
}

#[test]
fn env_key_from_args() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
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
    );
    yage!(
        "env",
        "--key",
        read(&key_path).trim(),
        &encrypted_path,
        "env"
    )
    .stdout(contains("foo=bar"))
    .stderr(is_empty());
    // again with short option
    yage!("env", "-k", read(&key_path).trim(), &encrypted_path, "env")
        .stdout(contains("foo=bar"))
        .stderr(is_empty());
}

#[test]
fn env_key_from_env() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
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
    );
    yage_cmd!("env", &encrypted_path, "env")
        .env("YAGE_KEY", read(&key_path).trim())
        .assert()
        .success()
        .stdout(contains("foo=bar"))
        .stderr(is_empty());
}

#[test]
fn env_key_file_from_env() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
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
    );
    yage_cmd!("env", &encrypted_path, "env")
        .env("YAGE_KEY_FILE", &key_path)
        .assert()
        .success()
        .stdout(contains("foo=bar"))
        .stderr(is_empty());
}

#[test]
fn env_key_from_stdin() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
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
    );
    yage_cmd!("env", "-K", "-", &encrypted_path, "env")
        .write_stdin(read(&key_path))
        .assert()
        .success()
        .stdout(contains("foo=bar"))
        .stderr(is_empty());
}

#[test]
fn env_empty() {
    yage_cmd!("env")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(
            "error: the following required arguments were not provided",
        ));
}

#[test]
fn env_no_command() {
    yage_cmd!("env", "foo.yaml")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(
            "error: the following required arguments were not provided",
        ));
}
