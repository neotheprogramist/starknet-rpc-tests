pub mod test_add_declare_v2;
pub mod test_add_declare_v3;
pub mod test_add_declare_v4;
use std::path::PathBuf;

pub struct TestSuiteCommonNested {
    pub test_path: PathBuf,
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(
    env!("OUT_DIR"),
    "/generated_tests_suite_common_suite_common_nested.rs"
));
