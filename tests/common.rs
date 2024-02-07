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
    fn assert(&self, predicate: impl predicates::Predicate<str>) -> &Self;
}

impl TestString for str {
    fn assert(&self, predicate: impl predicates::Predicate<str>) -> &Self {
        if let Some(case) = predicate.find_case(false, self.as_ref()) {
            panic!("{}", case.tree(),);
        }
        self
    }
}

impl TestString for String {
    fn assert(&self, predicate: impl predicates::Predicate<str>) -> &Self {
        if let Some(case) = predicate.find_case(false, self.as_ref()) {
            panic!("{}", case.tree(),);
        }
        self
    }
}

#[macro_export]
macro_rules! yage {
    () => (
        Command::cargo_bin("yage").unwrap().assert()
    );
    ( $( $v:expr ),* ) => (
        yage_args!(Command::cargo_bin("yage").unwrap(), $($v),*).assert()
    );
}

#[macro_export]
macro_rules! yage_args {
    ($x:expr) => ($x);
    ($x:expr, $y:expr) => ($x.arg($y));
    ($x:expr, $y:expr, $($z:expr),+) => (
        yage_args!($x.arg($y), $($z),*)
    );
}
