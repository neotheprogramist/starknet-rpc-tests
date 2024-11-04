pub mod test_add_declare_v2;
pub mod test_add_declare_v3;
use std::path::PathBuf;

pub struct TestSuite {
    pub test_path: PathBuf,
}

// Definicja głównego traitu
pub trait TestSuiteCommon {
    fn run(&self);
}

// Wczytywanie wygenerowanego kodu
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
