mod common;

use crate::common::*;
use assert_fs::prelude::*;
// use lipsum::lipsum;
// use predicates::prelude::predicate::path::*;
use predicates::prelude::predicate::str::*;
// use predicates::prelude::*;

#[test]
fn keygen_stdout() {
    yage!("keygen")
        .stdout(is_private_key())
        .stderr(is_pub_key_info());
}

#[test]
fn keygen_stdout_quiet() {
    yage!("keygen", "-q")
        .stdout(is_private_key())
        .stderr(is_empty());
}

#[test]
fn keygen_to_key_file() {
    let tmp = temp_dir();
    let key_path = tmp.child("private.key");
    yage!("keygen", "--output", &key_path)
        .stdout(is_empty())
        .stderr(is_pub_key_info());
    read(&key_path).assert(is_private_key());
    // again with the short option
    let key_path = tmp.child("private2.key");
    yage!("keygen", "-o", &key_path)
        .stdout(is_empty())
        .stderr(is_pub_key_info());
    read(&key_path).assert(is_private_key());
}

#[test]
fn keygen_to_public_file() {
    let tmp = temp_dir();
    let public_path = tmp.child("private.pub");
    yage!("keygen", "--public", &public_path)
        .stdout(is_private_key())
        .stderr(is_pub_key_info());
    read(&public_path).assert(is_public_key());
    // again with the short option
    let public_path = tmp.child("private2.pub");
    yage!("keygen", "-p", &public_path)
        .stdout(is_private_key())
        .stderr(is_pub_key_info());
    read(&public_path).assert(is_public_key());
}

#[test]
fn keygen_override_key_file() {
    let tmp = temp_dir();
    let key_path = tmp.child("private.key");
    key_path.touch().unwrap();
    yage_cmd!("keygen", "--output", &key_path)
        .assert()
        .failure()
        .stdout(is_empty())
        .stderr(contains(": File exists"));
}
