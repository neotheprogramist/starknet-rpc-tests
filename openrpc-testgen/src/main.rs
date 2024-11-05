fn main() {
    {
        use openrpc_testgen::suite_common::{TestSuite, TestSuiteCommon};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./common/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::suite_madara::{TestSuite, TestSuiteMadara};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./madara/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::suite_katana::{TestSuite, TestSuiteKatana};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./katana/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::suite_katana_specific::{TestSuite, TestSuiteKatanaSpecific};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./katana/`"),
        };

        suite.run();
    }
}
