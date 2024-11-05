use openrpc_testgen::suite_common::suite_common_nested::TestSuiteCommonNested;
use openrpc_testgen::suite_common::TestSuiteCommon;
use openrpc_testgen::suite_katana::TestSuiteKatana;
use openrpc_testgen::suite_madara::TestSuiteMadara;
use openrpc_testgen::RunnableTrait;
use std::path::PathBuf;

fn main() {
    let suite_common = TestSuiteCommon {
        test_path: PathBuf::from("./suite_common/"),
    };
    suite_common.run();

    let suite_madara = TestSuiteMadara {
        test_path: PathBuf::from("./suite_madara/"),
    };
    suite_madara.run();

    let suite_katana = TestSuiteKatana {
        test_path: PathBuf::from("./suite_katana/"),
    };
    suite_katana.run();

    let suite_common_nested = TestSuiteCommonNested {
        test_path: PathBuf::from("./suite_common/suite_common_nested/"),
    };
    suite_common_nested.run();
}
