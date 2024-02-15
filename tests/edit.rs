mod common;

use assert_fs::fixture::PathChild;
use common::*;
use predicates::str::{contains, is_empty};
use pretty_assertions::{assert_eq, assert_ne};
use serde_yaml as sy;
use yage::{check_encrypted, EncryptionStatus};

// editor command that add "hop: hop" to the file
#[cfg(windows)]
const EDITOR: &str = "cmd /c 'echo hop: hop >> %0'";
#[cfg(not(windows))]
const EDITOR: &str = "bash -c 'echo hop: hop >> $0'";

#[cfg(not(windows))]
#[test]
fn edit_key_file_from_args() {
    let (_tmp, key_path, _, _, encrypted_path) = generate_encrypted_file();
    let before_edit_data = read(&encrypted_path);
    yage!("edit", "--editor", EDITOR, "--key-file", &key_path, &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());
    let after_edit_data = read(&encrypted_path);
    assert_ne!(before_edit_data, after_edit_data);
    assert!(after_edit_data.starts_with(&before_edit_data));
    assert_eq!(
        check_encrypted(&sy::from_str(&after_edit_data).unwrap()),
        EncryptionStatus::Encrypted
    );
}

#[cfg(not(windows))]
#[test]
fn edit_key_file_from_env() {
    let (_tmp, key_path, _, _, encrypted_path) = generate_encrypted_file();
    let before_edit_data = read(&encrypted_path);
    yage_cmd!("edit", &encrypted_path)
        .env("EDITOR", EDITOR)
        .env("YAGE_KEY_FILE", &key_path)
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let after_edit_data = read(&encrypted_path);
    assert_ne!(before_edit_data, after_edit_data);
    assert!(after_edit_data.starts_with(&before_edit_data));
    assert_eq!(
        check_encrypted(&sy::from_str(&after_edit_data).unwrap()),
        EncryptionStatus::Encrypted
    );
}

#[cfg(not(windows))]
#[test]
fn edit_key_from_env() {
    let (_tmp, key_path, _, _, encrypted_path) = generate_encrypted_file();
    let before_edit_data = read(&encrypted_path);
    yage_cmd!("edit", "-e", EDITOR, &encrypted_path)
        .env("YAGE_KEY", read(&key_path).trim())
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let after_edit_data = read(&encrypted_path);
    assert_ne!(before_edit_data, after_edit_data);
    assert!(after_edit_data.starts_with(&before_edit_data));
    assert_eq!(
        check_encrypted(&sy::from_str(&after_edit_data).unwrap()),
        EncryptionStatus::Encrypted
    );
}

#[cfg(not(windows))]
#[test]
fn edit_key_from_stdin() {
    let (_tmp, key_path, _, _, encrypted_path) = generate_encrypted_file();
    let before_edit_data = read(&encrypted_path);
    yage_cmd!("edit", "-K", "-", "-e", EDITOR, &encrypted_path)
        .write_stdin(read(&key_path))
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let after_edit_data = read(&encrypted_path);
    assert_ne!(before_edit_data, after_edit_data);
    assert!(after_edit_data.starts_with(&before_edit_data));
    assert_eq!(
        check_encrypted(&sy::from_str(&after_edit_data).unwrap()),
        EncryptionStatus::Encrypted
    );
}

#[test]
fn edit_empty() {
    yage_cmd!("edit")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: the following required arguments were not provided"));
}

#[test]
fn edit_no_recipient() {
    let tmp = temp_dir();
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, "");
    yage_cmd!("edit", &yaml_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: no recipients provided"));
}
