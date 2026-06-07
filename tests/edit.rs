mod common;

use assert_fs::prelude::*;
use common::*;
use predicates::str::{contains, is_empty};
use pretty_assertions::{assert_eq, assert_ne};
use yage::{EncryptionStatus, check_encrypted};

// editor command that add "hop: hop" to the file
#[cfg(windows)]
const EDITOR: &str = "cmd /c 'echo hop: hop >> %0'";
#[cfg(not(windows))]
const EDITOR: &str = "bash -c 'echo hop: hop >> $0'";

#[cfg(not(windows))]
#[test]
fn edit_emits_warning_on_high_entropy_comment() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT_WITH_HIGH_ENTROPY_COMMENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!("encrypt", "-R", &pub_path, &yaml_path, "-o", &encrypted_path, "-qq")
        .stdout(is_empty())
        .stderr(is_empty());
    yage_cmd!("edit", "-K", &key_path, "--editor", EDITOR, &encrypted_path, "-q")
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(contains("high-entropy token detected"));
}

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
    assert_eq!(check_encrypted(&parse_yaml(&after_edit_data)), EncryptionStatus::Encrypted);
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
    assert_eq!(check_encrypted(&parse_yaml(&after_edit_data)), EncryptionStatus::Encrypted);
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
    assert_eq!(check_encrypted(&parse_yaml(&after_edit_data)), EncryptionStatus::Encrypted);
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
    assert_eq!(check_encrypted(&parse_yaml(&after_edit_data)), EncryptionStatus::Encrypted);
}

#[test]
fn edit_empty() {
    yage_cmd!("edit")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: the following required arguments were not provided"));
}
