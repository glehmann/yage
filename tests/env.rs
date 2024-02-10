mod common;

use assert_fs::fixture::PathChild;
use common::*;
use predicates::str::{contains, is_empty};

const YAML_CONTENT: &str = "foo: bar";

#[test]
fn env() {
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
    yage!("env", "-K", &key_path, &encrypted_path, "env")
        .stdout(contains("foo=bar"))
        .stderr(is_empty());
}
