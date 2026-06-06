mod common;

use common::*;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures")).join(name)
}

#[test]
fn decrypt_pre_computed() {
    let encrypted_path = fixture("encrypted_simple.yaml");
    let key_path = fixture("identity.age");
    let expected_path = fixture("plaintext_simple.yaml");
    let output = yage!("decrypt", "--key-file", key_path, encrypted_path).get_output().clone();
    assert_eq!(String::from_utf8(output.stdout).unwrap(), read(&expected_path));
}
