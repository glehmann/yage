pub trait TestPathChild {
    fn mkdir_all(&self) -> std::io::Result<()>;
}

pub trait TestToString {
    fn to_string(&self) -> String;
}

impl TestPathChild for assert_fs::fixture::ChildPath {
    fn mkdir_all(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.path())
    }
}

impl TestToString for assert_fs::fixture::ChildPath {
    fn to_string(&self) -> String {
        self.path().display().to_string()
    }
}

impl TestToString for assert_fs::TempDir {
    fn to_string(&self) -> String {
        self.path().display().to_string()
    }
}

#[macro_export]
macro_rules! yage {
    ( $( $v:expr ),* ) => (
        {
            let temp_vec: Vec<String> = vec![$($v.to_string(),)*];
            Command::cargo_bin("yage").unwrap().args(&temp_vec).assert()
        }
    );
}
