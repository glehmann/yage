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
    yage!(
        "edit",
        "--editor",
        EDITOR,
        "--key-file",
        &key_path,
        &encrypted_path
    )
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
        .stderr(contains(
            "error: the following required arguments were not provided",
        ));
}

#[cfg(not(windows))]
#[test]
fn edit_multiple_recipients() {
    let tmp = temp_dir();
    let (key_path0, pub_path0) = create_key(&tmp);
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let (key_path4, pub_path4) = create_key(&tmp);
    let (key_path5, pub_path5) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!(
        "encrypt",
        "-R",
        &pub_path0,
        "-R",
        &pub_path1,
        "-R",
        &pub_path2,
        "-R",
        &pub_path3,
        "-R",
        &pub_path4,
        "-R",
        &pub_path5,
        &yaml_path,
        "-o",
        &encrypted_path
    );
    let (key_path6, _) = create_key(&tmp);
    let (key_path7, _) = create_key(&tmp);
    let before_edit_data = read(&encrypted_path);
    yage_cmd!(
        "edit",
        "-e",
        EDITOR,
        "-K",
        key_path0,
        "--recipient",
        read(&pub_path1).trim(),
        "--recipient-file",
        &pub_path2,
        "-r",
        read(&pub_path3).trim(),
        "-R",
        "-",
        &encrypted_path
    )
    .env("YAGE_RECIPIENT", read(&key_path6).trim())
    .env("YAGE_RECIPIENT_FILE", &key_path7)
    .write_stdin(format!("{}{}", read(&pub_path4), read(&pub_path5)))
    .assert()
    .success()
    .stdout(is_empty())
    .stderr(is_empty());
    let data: sy::Value = sy::from_str(YAML_CONTENT).unwrap();
    let after_edit_data = read(&encrypted_path);
    let encrypted_data: sy::Value = sy::from_str(&read(&encrypted_path)).unwrap();
    assert!(after_edit_data.starts_with(&before_edit_data));
    for key_path in [
        key_path0, key_path1, key_path2, key_path3, key_path4, key_path5,
    ] {
        let identities = yage::load_identities(&[], &[key_path]).unwrap();
        let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
        assert_ne!(data, decrypted_data);
    }
    // YAGE_RECIPIENT env is overridden by command line
    let identities = yage::load_identities(&[], &[key_path6]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data, &identities).is_err());
    // YAGE_RECIPIENT_FILE env is overridden by command line
    let identities = yage::load_identities(&[], &[key_path7]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data, &identities).is_err());
}
