mod common;

use assert_fs::prelude::*;
use predicates::prelude::predicate::str::*;
use pretty_assertions::assert_eq;
use serde_yaml as sy;

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
fn re_encrypt_to_stdout() {
    let (_tmp, key_path, pub_path, yaml_path, encrypted_path) = generate_encrypted_file();
    let output = yage!("re-encrypt", "-K", key_path, "-R", &pub_path, &encrypted_path)
        .stdout(is_match(YAML_CONTENT_ENCRYPTED_PATTERN).unwrap())
        .stderr(is_empty())
        .get_output()
        .clone();
    let data: sy::Value = yage::read_yaml(&yaml_path).unwrap();
    let encrypted_data: sy::Value = yage::read_yaml(&encrypted_path).unwrap();
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let re_encrypted_data: sy::Value =
        sy::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
    assert_ne!(encrypted_data, re_encrypted_data);
}

#[test]
fn re_encrypt_to_file() {
    let (tmp, key_path, pub_path, yaml_path, encrypted_path) = generate_encrypted_file();
    let re_encrypted_path = tmp.child("file.re-enc.yaml");
    yage!("re-encrypt", "-K", key_path, "-R", &pub_path, &encrypted_path, "-o", &re_encrypted_path)
        .stdout(is_empty())
        .stderr(is_empty());
    let data: sy::Value = yage::read_yaml(&yaml_path).unwrap();
    let encrypted_data: sy::Value = yage::read_yaml(&encrypted_path).unwrap();
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let re_encrypted_data: sy::Value = sy::from_str(&read(&re_encrypted_path)).unwrap();
    let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
    assert_ne!(encrypted_data, re_encrypted_data);
}

#[test]
fn re_encrypt_keep_recipients() {
    let (tmp, key_path, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let re_encrypted_path = tmp.child("file.re-enc.yaml");
    yage!(
        "re-encrypt",
        "-K",
        key_path,
        "--keep-recipients",
        &encrypted_path,
        "-o",
        &re_encrypted_path
    )
    .stdout(is_empty())
    .stderr(is_empty());
    let data: sy::Value = yage::read_yaml(&yaml_path).unwrap();
    let encrypted_data: sy::Value = yage::read_yaml(&encrypted_path).unwrap();
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let re_encrypted_data: sy::Value = sy::from_str(&read(&re_encrypted_path)).unwrap();
    let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
    assert_ne!(encrypted_data, re_encrypted_data);
}

#[test]
fn re_encrypt_key_stdin() {
    let (tmp, key_path, pub_path, yaml_path, encrypted_path) = generate_encrypted_file();
    let re_encrypted_path = tmp.child("file.re-enc.yaml");
    yage_cmd!("re-encrypt", "-K", "-", "-R", &pub_path, &encrypted_path, "-o", &re_encrypted_path)
        .write_stdin(read(&key_path))
        .assert()
        .success()
        .stdout(is_empty())
        .stderr(is_empty());
    let data: sy::Value = yage::read_yaml(&yaml_path).unwrap();
    let encrypted_data: sy::Value = yage::read_yaml(&encrypted_path).unwrap();
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let re_encrypted_data: sy::Value = sy::from_str(&read(&re_encrypted_path)).unwrap();
    let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
    assert_ne!(encrypted_data, re_encrypted_data);
}

#[test]
fn re_encrypt_no_recipient() {
    let (_tmp, key_path, _, _, encrypted_path) = generate_encrypted_file();
    yage_cmd!("re-encrypt", "-K", key_path, &encrypted_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(""));
}

#[test]
fn re_encrypt_no_key() {
    let (_tmp, _, pub_path, _, encrypted_path) = generate_encrypted_file();
    yage_cmd!("re-encrypt", "-R", pub_path, &encrypted_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: no keys provided"));
}

#[test]
fn re_encrypt_multiple_recipients() {
    let (tmp, key_path1, pub_path1, yaml_path, encrypted_path) = generate_encrypted_file();
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let (key_path4, pub_path4) = create_key(&tmp);
    let (key_path5, pub_path5) = create_key(&tmp);
    let (key_path6, _) = create_key(&tmp);
    let (key_path7, _) = create_key(&tmp);
    let re_encrypted_path = tmp.child("file.re-enc.yaml");
    yage_cmd!(
        "re-encrypt",
        "--key-file",
        &key_path1,
        "--recipient",
        read(&pub_path1).trim(),
        "--recipient-file",
        &pub_path2,
        "-r",
        read(&pub_path3).trim(),
        "-R",
        "-",
        &encrypted_path,
        "--output",
        &re_encrypted_path
    )
    .env("YAGE_RECIPIENT", read(&key_path6).trim())
    .env("YAGE_RECIPIENT_FILE", &key_path7)
    .write_stdin(format!("{}{}", read(&pub_path4), read(&pub_path5)))
    .assert()
    .success()
    .stdout(is_empty())
    .stderr(is_empty());
    let data = yage::read_yaml(&yaml_path).unwrap();
    let re_encrypted_data: sy::Value = sy::from_str(&read(&re_encrypted_path)).unwrap();
    for key_path in [key_path1, key_path2, key_path3, key_path4, key_path5] {
        let identities = yage::load_identities(&[], &[key_path]).unwrap();
        let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
        assert_eq!(data, decrypted_data);
    }
    // YAGE_RECIPIENT env is overridden by command line
    let identities = yage::load_identities(&[], &[key_path6]).unwrap();
    assert!(yage::decrypt_yaml(&re_encrypted_data, &identities).is_err());
    // YAGE_RECIPIENT_FILE env is overridden by command line
    let identities = yage::load_identities(&[], &[key_path7]).unwrap();
    assert!(yage::decrypt_yaml(&re_encrypted_data, &identities).is_err());
}

#[test]
fn re_encrypt_recipients_from_env() {
    let (tmp, key_path1, pub_path1, yaml_path, encrypted_path) = generate_encrypted_file();
    let (key_path2, pub_path2) = create_key(&tmp);
    let (key_path3, pub_path3) = create_key(&tmp);
    let (key_path4, pub_path4) = create_key(&tmp);
    let re_encrypted_path = tmp.child("file.re-enc.yaml");
    yage_cmd!(
        "re-encrypt",
        "-k",
        read(&key_path1).trim(),
        &encrypted_path,
        "--output",
        &re_encrypted_path
    )
    .env("YAGE_RECIPIENT", format!("{},{}", read(&pub_path1).trim(), read(&pub_path2).trim()))
    .env("YAGE_RECIPIENT_FILE", std::env::join_paths(vec![&pub_path3, &pub_path4]).unwrap())
    .assert()
    .success()
    .stdout(is_empty())
    .stderr(is_empty());
    let data = yage::read_yaml(&yaml_path).unwrap();
    let re_encrypted_data: sy::Value = sy::from_str(&read(&re_encrypted_path)).unwrap();
    for key_path in [key_path1, key_path2, key_path3, key_path4] {
        let identities = yage::load_identities(&[], &[key_path]).unwrap();
        let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
        assert_eq!(data, decrypted_data);
    }
}

#[test]
fn re_encrypt_from_stdin() {
    let (tmp, key_path, _, yaml_path, encrypted_path) = generate_encrypted_file();
    let re_encrypted_path = tmp.child("file.re-enc.yaml");
    yage_cmd!(
        "re-encrypt",
        "--key",
        read(&key_path).trim(),
        "-e",
        "-",
        "--output",
        &re_encrypted_path
    )
    .write_stdin(read(&encrypted_path))
    .assert()
    .success()
    .stdout(is_empty())
    .stderr(is_empty());
    let data = yage::read_yaml(&yaml_path).unwrap();
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    let re_encrypted_data: sy::Value = sy::from_str(&read(&re_encrypted_path)).unwrap();
    let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
    assert_eq!(data, decrypted_data);
}

#[test]
fn re_encrypt_in_place() {
    let (tmp, key_path, pub_path, yaml_path, encrypted_path) = generate_encrypted_file();
    let other_path = tmp.child("other.yaml");
    write(&other_path, &read(&encrypted_path));
    yage!("re-encrypt", "-K", key_path, "-R", &pub_path, "-i", &encrypted_path, &other_path)
        .stdout(is_empty())
        .stderr(is_empty());
    let data = yage::read_yaml(&yaml_path).unwrap();
    let identities = yage::load_identities(&[], &[key_path]).unwrap();
    for path in [&encrypted_path, other_path.path()] {
        let re_encrypted_data = yage::read_yaml(path).unwrap();
        let decrypted_data = yage::decrypt_yaml(&re_encrypted_data, &identities).unwrap();
        assert_eq!(data, decrypted_data);
    }
}

#[test]
fn re_encrypt_stdin_in_place() {
    yage_cmd!("re-encrypt", "--in-place", "-")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: stdin can't be modified in place"));
}

#[test]
fn re_encrypt_multiple_files_no_in_place() {
    yage_cmd!("re-encrypt", "foo.yaml", "bar.yaml")
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains("error: invalid number of input files"));
}
