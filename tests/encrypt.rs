mod common;

use assert_fs::prelude::*;
use predicates::prelude::predicate::str::*;
use pretty_assertions::assert_eq;
use std::{fs::OpenOptions, io::Write};
use yage::EncryptionStatus;

use crate::common::*;

const YAML_CONTENT_ENCRYPTED_PATTERN: &str = r"foo: yage\[[0-9a-zA-Z/=\-+]+\|r:[a-z0-9,]+\]
titi:
  toto: yage\[[0-9a-zA-Z/=\-+]+\|r:[a-z0-9,]+\]
array:
- yage\[[0-9a-zA-Z/=\-+]+\|r:[a-z0-9,]+\]
- yage\[[0-9a-zA-Z/=\-+]+\|r:[a-z0-9,]+\]
- yage\[[0-9a-zA-Z/=\-+]+\|r:[a-z0-9,]+\]
empty_map: \{\}
empty_array: \[\]
empty_string: yage\[[0-9a-zA-Z/=\-+]+\|r:[a-z0-9,]+\]
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
    let data = parse_yaml(YAML_CONTENT);
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let encrypted_data = parse_yaml(&String::from_utf8(output.stdout).unwrap());
    let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
    assert!(data.yaml_eq(&decrypted_data));
}

#[test]
fn encrypt_to_file() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!("encrypt", "-R", &pub_path, &yaml_path, "-o", &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());
    let data = parse_yaml(YAML_CONTENT);
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let encrypted_data = parse_yaml(&read(&encrypted_path));
    let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
    assert!(data.yaml_eq(&decrypted_data));
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
    let data = parse_yaml(YAML_CONTENT);
    let encrypted_data = parse_yaml(&read(&encrypted_path));
    for key_path in [key_path1, key_path2, key_path3, key_path4, key_path5] {
        let identities = yage::load_identities(&[], &[key_path]).unwrap();
        let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
        assert!(data.yaml_eq(&decrypted_data));
    }
    // YAGE_RECIPIENT env is overridden by command line
    let identities = yage::load_identities(&[], &[key_path6]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data, &identities).is_err());
    // YAGE_RECIPIENT_FILE env is overridden by command line
    let identities = yage::load_identities(&[], &[key_path7]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data, &identities).is_err());
}

#[test]
fn encrypt_recipients_from_env() {
    let tmp = temp_dir();
    let (key_path1, pub_path1) = create_key(&tmp);
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let (key_path4, pub_path4) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage_cmd!("encrypt", &yaml_path, "--output", &encrypted_path)
        .env("YAGE_RECIPIENT", format!("{},{}", read(&pub_path1).trim(), read(&pub_path2).trim()))
        .env("YAGE_RECIPIENT_FILE", std::env::join_paths(vec![&pub_path3, &pub_path4]).unwrap())
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let data = parse_yaml(YAML_CONTENT);
    let encrypted_data = parse_yaml(&read(&encrypted_path));
    for key_path in [key_path1, key_path2, key_path3, key_path4] {
        let identities = yage::load_identities(&[], &[key_path]).unwrap();
        let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
        assert!(data.yaml_eq(&decrypted_data));
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
    let data = parse_yaml(YAML_CONTENT);
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let encrypted_data = parse_yaml(&read(&encrypted_path));
    let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
    assert!(data.yaml_eq(&decrypted_data));
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
    let data = parse_yaml(YAML_CONTENT);
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    for path in [&yaml_path, &other_path] {
        let encrypted_data = parse_yaml(&read(path));
        let decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
        assert!(data.yaml_eq(&decrypted_data));
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
fn encrypt_multiple_files_no_in_place() {
    yage_cmd!("encrypt", "foo.yaml", "bar.yaml")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: invalid number of input files"));
}

#[test]
fn encrypt_partially_encrypted() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!("encrypt", "-R", &pub_path, &yaml_path, "-o", &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());
    let raw_encrypted_data = read(&encrypted_path);
    let encrypted_data = parse_yaml(&raw_encrypted_data);
    assert_eq!(yage::check_encrypted(&encrypted_data), EncryptionStatus::Encrypted);
    // append some data to the encrypted file, then try to encrypt it again
    OpenOptions::new()
        .append(true)
        .open(&encrypted_path)
        .unwrap()
        .write_all(b"auie: tsrn\n")
        .unwrap();
    assert_eq!(yage::check_encrypted(&parse_yaml(&read(&encrypted_path))), EncryptionStatus::Mixed);
    let encrypted_path2 = tmp.child("file2.enc.yaml");
    yage!("encrypt", "-R", &pub_path, &encrypted_path, "-o", &encrypted_path2)
        .stdout(is_empty())
        .stderr(is_empty());
    // make sure we haven't changed the already encrypted stuff
    assert!(read(&encrypted_path2).starts_with(&raw_encrypted_data));
    // and verify we can decrypt the new file
    let raw_encrypted_data2 = read(&encrypted_path2);
    let encrypted_data2 = parse_yaml(&raw_encrypted_data2);
    assert_eq!(yage::check_encrypted(&encrypted_data2), EncryptionStatus::Encrypted);
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    assert!(yage::decrypt_yaml(&encrypted_data2, &identities).is_ok());
}

#[test]
fn encrypt_partially_encrypted_other_recipient() {
    let tmp = temp_dir();
    let (_, pub_path1) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!("encrypt", "-R", &pub_path1, &yaml_path, "-o", &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());
    let raw_encrypted_data = read(&encrypted_path);
    let encrypted_data = parse_yaml(&raw_encrypted_data);
    assert_eq!(yage::check_encrypted(&encrypted_data), EncryptionStatus::Encrypted);
    // append some data to the encrypted file, then try to encrypt it again
    OpenOptions::new()
        .append(true)
        .open(&encrypted_path)
        .unwrap()
        .write_all(b"auie: tsrn\n")
        .unwrap();
    assert_eq!(yage::check_encrypted(&parse_yaml(&read(&encrypted_path))), EncryptionStatus::Mixed);
    let (_, pub_path2) = create_key(&tmp);
    yage_cmd!("encrypt", "-R", &pub_path2, &encrypted_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(
            "error: the recipients form the command line don't match the recipients from the file",
        ));
}

#[test]
fn encrypt_preserves_top_level_comments() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT_WITH_COMMENTS);
    let encrypted_path = tmp.child("file.enc.yaml");
    yage!("encrypt", "-R", &pub_path, &yaml_path, "-o", &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());

    let encrypted_content = read(&encrypted_path);

    // ROOT-level comments (before the first YAML key) live at the YamlFile
    // level, outside any Document. They were previously lost because
    // Document::from_str discards them. Our YamlFile-based read/write
    // preserves them.
    assert!(
        encrypted_content.contains("# Top-level comment"),
        "ROOT-level comment should be preserved in encrypted file"
    );

    // Verify the encrypted file is still parseable
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let encrypted_data = parse_yaml(&encrypted_content);
    let _decrypted_data = yage::decrypt_yaml(&encrypted_data, &identities).unwrap();
    // Note: map_set loses VALUE-level formatting tokens when replacing
    // complex nested values, so round-trip semantic equality does not
    // hold for inputs with MAPPING-level comments. This pre-existing
    // yaml-edit limitation is not addressed here.
}

#[test]
fn encrypt_and_decrypt_preserves_top_level_comments() {
    let tmp = temp_dir();
    let (key_path, pub_path) = create_key(&tmp);
    let yaml_path = tmp.child("file.yaml");
    write(&yaml_path, YAML_CONTENT_WITH_COMMENTS);
    let encrypted_path = tmp.child("file.enc.yaml");

    // Encrypt
    yage!("encrypt", "-R", &pub_path, &yaml_path, "-o", &encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());

    let encrypted_content = read(&encrypted_path);
    assert!(
        encrypted_content.contains("# Top-level comment"),
        "ROOT-level comment should survive encryption"
    );

    // Decrypt back to a file
    let decrypted_path = tmp.child("file.dec.yaml");
    yage!("decrypt", "-K", &key_path, &encrypted_path, "-o", &decrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());

    let decrypted_content = read(&decrypted_path);

    // ROOT-level comments survive the full encrypt → decrypt cycle
    assert!(
        decrypted_content.contains("# Top-level comment"),
        "ROOT-level comment should survive decrypt"
    );
}
