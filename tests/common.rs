use std::ops::Deref;

use predicates_tree::CaseTreeExt;

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
