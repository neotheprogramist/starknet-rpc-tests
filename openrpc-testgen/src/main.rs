fn main() {
    {
        use openrpc_testgen::suite_common::{TestSuite, TestSuiteCommon};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./suite_common/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::suite_madara::{TestSuite, TestSuiteMadara};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./suite_madara/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::suite_katana::{TestSuite, TestSuiteKatana};
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./suite_katana/`"),
        };

        suite.run();
    }
    {
        use openrpc_testgen::suite_common::suite_common_nested::{
            TestSuite, TestSuiteCommonNested,
        };
        let suite = TestSuite {
            test_path: std::path::PathBuf::from("./suite_common_suite_common_nested"),
        };
        suite.run();
    }
}
