mod common;

use crate::common::*;
use assert_fs::prelude::*;
use predicates::str::{contains, is_empty};
use pretty_assertions::assert_eq;

#[test]
fn recipients_to_stdout() {
    let (_tmp, _, pub_path1, _, encrypted_path1) = generate_encrypted_file();
    let (_tmp, _, pub_path2, _, encrypted_path2) = generate_encrypted_file();
    yage!("recipients", &encrypted_path1, &encrypted_path2)
        .stdout(contains(format!("{}: {}", encrypted_path1.to_string_lossy(), read(&pub_path1))))
        .stdout(contains(format!("{}: {}", encrypted_path2.to_string_lossy(), read(&pub_path2))))
        .stderr(is_empty());
}

#[test]
fn recipients_to_file() {
    let (_tmp, _, pub_path1, _, encrypted_path1) = generate_encrypted_file();
    let (tmp, _, pub_path2, _, encrypted_path2) = generate_encrypted_file();
    let recipients_path = tmp.child("recipients.txt");
    yage!("recipients", &encrypted_path1, &encrypted_path2, "--output", &recipients_path)
        .stdout(is_empty())
        .stderr(is_empty());
    assert_eq!(
        read(&recipients_path),
        format!(
            "{}: {}{}: {}",
            encrypted_path1.to_string_lossy(),
            read(&pub_path1),
            encrypted_path2.to_string_lossy(),
            read(&pub_path2)
        )
    );
}

#[test]
fn recipients_only_recipients() {
    let (_tmp, _, pub_path1, _, encrypted_path1) = generate_encrypted_file();
    let (tmp, _, pub_path2, _, encrypted_path2) = generate_encrypted_file();
    let recipients_path = tmp.child("recipients.txt");
    yage!(
        "recipients",
        &encrypted_path1,
        &encrypted_path2,
        &encrypted_path1,
        &encrypted_path2,
        "-o",
        &recipients_path,
        "--only-recipients"
    )
    .stdout(is_empty())
    .stderr(is_empty());
    let recipient1 = read(&pub_path1);
    let recipient2 = read(&pub_path2);
    if recipient1 < recipient2 {
        assert_eq!(read(&recipients_path), format!("{recipient1}{recipient2}"));
    } else {
        assert_eq!(read(&recipients_path), format!("{recipient2}{recipient1}"));
    }
}
