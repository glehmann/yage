#![allow(dead_code)]

use std::{
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
};

use assert_fs::fixture::ChildPath;
use assert_fs::{prelude::*, TempDir};
use predicates::prelude::predicate::str::*;
use predicates_tree::CaseTreeExt;

pub const KEY_PATTERN: &str = r"^AGE-SECRET-KEY-[0-9A-Z]{59}\s*$";
pub const PUBKEY_PATTERN: &str = r"^[0-9a-z]{62}\s*$";
pub const PUBKEY_INFO_PATTERN: &str = r"^Public key: [0-9a-z]{62}\s+$";

pub trait TestPathChild {
    fn mkdir_all(&self) -> std::io::Result<()>;
}

impl TestPathChild for assert_fs::fixture::ChildPath {
    fn mkdir_all(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.path())
    }
}

pub trait TestString {
    fn assert(&self, predicate: impl predicates::Predicate<str>) -> &Self
    where
        Self: Deref<Target = str>,
    {
        if let Some(case) = predicate.find_case(false, self) {
            panic!("{}", case.tree());
        }
        self
    }
}

impl TestString for str {}
impl TestString for String {}

#[macro_export]
macro_rules! yage {
    ( $( $v:expr ),* ) => ({
        use std::process::Command;
        use assert_cmd::prelude::*;
        let mut cmd = Command::cargo_bin("yage").unwrap();
        $(
            cmd.arg($v.to_cmd_arg());
        )*
        cmd.assert()
    });
}

pub trait ToCmdArg {
    fn to_cmd_arg(&self) -> &OsStr;
}

impl ToCmdArg for Path {
    fn to_cmd_arg(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl ToCmdArg for PathBuf {
    fn to_cmd_arg(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl ToCmdArg for str {
    fn to_cmd_arg(&self) -> &OsStr {
        OsStr::new(self)
    }
}

impl ToCmdArg for &'static str {
    fn to_cmd_arg(&self) -> &OsStr {
        OsStr::new(self)
    }
}

// to be able to path the various path types we have as function arguments

pub trait ToPath {
    fn path(&self) -> &Path;
}

impl ToPath for Path {
    fn path(&self) -> &Path {
        self
    }
}

impl ToPath for PathBuf {
    fn path(&self) -> &Path {
        self
    }
}

impl ToPath for ChildPath {
    fn path(&self) -> &Path {
        self.path()
    }
}

pub fn read(path: &dyn ToPath) -> String {
    std::fs::read_to_string(path.path()).unwrap()
}

pub fn temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

pub fn is_public_key() -> impl predicates::Predicate<str> {
    is_match(PUBKEY_PATTERN).unwrap()
}

pub fn is_private_key() -> impl predicates::Predicate<str> {
    is_match(KEY_PATTERN).unwrap()
}

pub fn is_pub_key_info() -> impl predicates::Predicate<str> {
    is_match(PUBKEY_INFO_PATTERN).unwrap()
}

pub fn create_key(tmp: &TempDir) -> (PathBuf, PathBuf) {
    let id = uuid::Uuid::new_v4();
    let key_path = tmp.child(format!("{}.key", id.to_string()));
    let public_path = tmp.child(format!("{}.pub", id.to_string()));
    yage!("keygen", "--output", &key_path, "--public", &public_path)
        .success()
        .stdout(is_empty())
        .stderr(is_pub_key_info());
    (key_path.path().into(), public_path.path().into())
}
