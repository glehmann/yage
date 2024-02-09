mod common;

use std::{fs::OpenOptions, io::Write};

use assert_fs::prelude::*;
use predicates::prelude::predicate::str::*;
use serde_yaml as sy;
use yage::EncryptionStatus;

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

const YAML_CONTENT_ENCRYPTED_PATTERN: &str = r"foo: yage\[[0-9a-zA-Z/=\-+]+\]
titi:
  toto: yage\[[0-9a-zA-Z/=\-+]+\]
array:
- yage\[[0-9a-zA-Z/=\-+]+\]
- yage\[[0-9a-zA-Z/=\-+]+\]
- yage\[[0-9a-zA-Z/=\-+]+\]
empty_map: \{\}
empty_array: \[\]
empty_string: yage\[[0-9a-zA-Z/=\-+]+\]
empty: null
";

#[test]
fn encrypt_to_stdout() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let output = yage!("encrypt", "-R", &pub_path, &yaml_path)
        .stdout(is_match(YAML_CONTENT_ENCRYPTED_PATTERN).unwrap())
        .stderr(is_empty())
        .get_output()
        .clone();
    let data: sy::Value = sy::from_str(&YAML_CONTENT).unwrap();
    let identities = yage::load_identities(&vec![], &vec![key_path]).unwrap();
    let encrypted_data: sy::Value =
        sy::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
}

#[test]
fn encrypt_to_file() {
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
    )
    .stdout(is_empty())
    .stderr(is_empty());
    let data: sy::Value = sy::from_str(&YAML_CONTENT).unwrap();
    let identities = yage::load_identities(&vec![], &vec![key_path]).unwrap();
    let encrypted_data: sy::Value = sy::from_str(&read(&encrypted_path)).unwrap();
    let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
}

#[test]
fn encrypt_no_recipient() {
    let tmp = temp_dir();
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    yage_cmd!("encrypt", &yaml_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: no recipients provided"));
}

#[test]
fn encrypt_multiple_recipients() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let (key_path4, pub_path4) = create_key(&tmp);
    let (key_path5, pub_path5) = create_key(&tmp);
    let (key_path6, _) = create_key(&tmp);
    let (key_path7, _) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage_cmd!(
        "encrypt",
        "--recipient",
        read(&pub_path1).trim(),
        "--recipient-file",
        &pub_path2,
        "-r",
        read(&pub_path3).trim(),
        "-R",
        "-",
        &yaml_path,
        "--output",
        &encrypted_path
    )
    .env("YAGE_RECIPIENT", read(&key_path6).trim())
    .env("YAGE_RECIPIENT_FILE", &key_path7)
    .write_stdin(format!("{}{}", read(&pub_path4), read(&pub_path5)))
    .assert()
    .success()
    .stdout(is_empty())
    .stderr(is_empty());
    let data: sy::Value = sy::from_str(&YAML_CONTENT).unwrap();
    let encrypted_data: sy::Value = sy::from_str(&read(&encrypted_path)).unwrap();
    for key_path in vec![key_path1, key_path2, key_path3, key_path4, key_path5] {
        let identities = yage::load_identities(&vec![], &vec![key_path]).unwrap();
        let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
        assert_eq!(data, decrypted_data);
    }
    // YAGE_RECIPIENT env is overridden by command line
    let identities = yage::load_identities(&vec![], &vec![key_path6]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data, &identities).is_err());
    // YAGE_RECIPIENT_FILE env is overridden by command line
    let identities = yage::load_identities(&vec![], &vec![key_path7]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data, &identities).is_err());
}

#[test]
fn encrypt_recipients_from_env() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage_cmd!("encrypt", &yaml_path, "--output", &encrypted_path)
        .env(
            "YAGE_RECIPIENT",
            format!("{},{}", read(&pub_path1).trim(), read(&pub_path2).trim()),
        )
        .env("YAGE_RECIPIENT_FILE", &pub_path3)
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let data: sy::Value = sy::from_str(&YAML_CONTENT).unwrap();
    let encrypted_data: sy::Value = sy::from_str(&read(&encrypted_path)).unwrap();
    for key_path in vec![key_path1, key_path2, key_path3] {
        let identities = yage::load_identities(&vec![], &vec![key_path]).unwrap();
        let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
        assert_eq!(data, decrypted_data);
    }
}

#[test]
fn encrypt_from_stdin() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage_cmd!("encrypt", "-R", &pub_path, "-", "--output", &encrypted_path)
        .write_stdin(YAML_CONTENT)
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let data: sy::Value = sy::from_str(&YAML_CONTENT).unwrap();
    let identities = yage::load_identities(&vec![], &vec![key_path]).unwrap();
    let encrypted_data: sy::Value = sy::from_str(&read(&encrypted_path)).unwrap();
    let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
}

#[test]
fn encrypt_in_place() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    let other_path = tmp.child("other.yaml");
    write(&yaml_path, YAML_CONTENT);
    write(&other_path, YAML_CONTENT);
    yage!("encrypt", "-R", &pub_path, "-i", &yaml_path, &other_path)
        .stdout(is_empty())
        .stderr(is_empty());
    let data: sy::Value = sy::from_str(&YAML_CONTENT).unwrap();
    let identities = yage::load_identities(&vec![], &vec![key_path]).unwrap();
    for path in vec![&yaml_path, &other_path] {
        let encrypted_data: sy::Value = sy::from_str(&read(path)).unwrap();
        let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
        assert_eq!(data, decrypted_data);
    }
}

#[test]
fn encrypt_stdin_in_place() {
    yage_cmd!("encrypt", "--in-place", "-")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: stdin can't be modified in place"));
}

#[test]
fn encrypt_partially_encrypted() {
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
    )
    .stdout(is_empty())
    .stderr(is_empty());
    let raw_encrypted_data = read(&encrypted_path);
    dbg!(&raw_encrypted_data);
    let encrypted_data: sy::Value = sy::from_str(&raw_encrypted_data).unwrap();
    assert_eq!(
        yage::check_encrypted(&encrypted_data),
        EncryptionStatus::Encrypted
    );
    // append some data to the encrypted file, then try to encrypt it again
    OpenOptions::new()
        .append(true)
        .open(&encrypted_path)
        .unwrap()
        .write_all(b"auie: tsrn\n")
        .unwrap();
    assert_eq!(
        yage::check_encrypted(&sy::from_str(&read(&encrypted_path)).unwrap()),
        EncryptionStatus::Mixed
    );
    let encrypted_path2 = tmp.child("file2.enc.yaml");
    yage!(
        "encrypt",
        "-R",
        &pub_path,
        &encrypted_path,
        "-o",
        &encrypted_path2
    )
    .stdout(is_empty())
    .stderr(is_empty());
    // make sure we haven't changed the already encrypted stuff
    assert!(read(&encrypted_path2).starts_with(&raw_encrypted_data));
    // and verify we can decrypt the new file
    let raw_encrypted_data2 = read(&encrypted_path2);
    let encrypted_data2: sy::Value = sy::from_str(&raw_encrypted_data2).unwrap();
    assert_eq!(
        yage::check_encrypted(&encrypted_data2),
        EncryptionStatus::Encrypted
    );
    let identities = yage::load_identities(&vec![], &vec![key_path]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data2, &identities).is_ok());
}
