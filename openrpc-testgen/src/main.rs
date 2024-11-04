// Importujemy `common::TestSuite`
use openrpc_testgen::common::{TestSuite, TestSuiteCommon};

fn main() {
    // Tworzymy instancję `TestSuite`
    let suite = TestSuite {
        test_path: std::path::PathBuf::from("./common/`"),
    };

    // Wywołujemy `run`, aby uruchomić wszystkie testy
    suite.run();
}
