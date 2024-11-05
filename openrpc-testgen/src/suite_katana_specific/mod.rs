pub mod test_add_declare_v2;
pub mod test_add_declare_v3;
pub mod test_add_declare_v4;
pub mod test_add_declare_v5;
use std::path::PathBuf;

pub struct TestSuite {
    pub test_path: PathBuf,
}

pub trait TestSuiteKatanaSpecific {
    fn run(&self);
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(
    env!("OUT_DIR"),
    "/generated_tests_suite_katana_specific.rs"
));
