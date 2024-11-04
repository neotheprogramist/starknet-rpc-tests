fn main() {
    {
        use openrpc_testgen::common::{TestSuite, TestSuiteCommon};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./common/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::madara::{TestSuite, TestSuiteMadara};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./madara/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::katana::{TestSuite, TestSuiteKatana};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./katana/`"),
        };

        suite.run();
    }
}
