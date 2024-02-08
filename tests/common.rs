use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use assert_fs::fixture::ChildPath;
use predicates_tree::CaseTreeExt;

#[allow(dead_code)]
pub const KEY_PATTERN: &str = r"^AGE-SECRET-KEY-[0-9A-Z]{59}\s*$";
#[allow(dead_code)]
pub const PUBKEY_PATTERN: &str = r"^[0-9a-z]{62}\s*$";
#[allow(dead_code)]
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
        let mut cmd = Command::cargo_bin("yage").unwrap();
        $(
            cmd.arg($v);
        )*
        cmd.assert()
    });
}

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

#[allow(dead_code)]
pub fn read(path: &dyn ToPath) -> String {
    std::fs::read_to_string(path.path()).unwrap()
}
