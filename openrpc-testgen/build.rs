use std::env;
use std::fs::{self, read_to_string, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let src_dir = Path::new("src");

    // Clear old generated files
    for entry in fs::read_dir(&out_dir).expect("Could not read OUT_DIR") {
        let entry = entry.expect("Could not read directory entry");
        let path = entry.path();
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with("generated_tests_"))
            == Some(true)
        {
            fs::remove_file(path).expect("Could not remove old generated test file");
        }
    }

    // Process each root suite directory in `src`
    for entry in fs::read_dir(src_dir).expect("Could not read src directory") {
        let entry = entry.expect("Could not read directory entry");
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("suite_"))
                == Some(true)
        {
            let root_output_type = process_module_directory(&path, &out_dir, None); // Root suite has no parent Output type
            process_directory_recursively(&path, &out_dir, Some(&root_output_type));
        }
    }

    println!("cargo:rerun-if-changed=src");
}

// Recursively processes directories with prefix "suite_"
fn process_directory_recursively(dir: &Path, out_dir: &str, parent_output_type: Option<&str>) {
    for entry in fs::read_dir(dir).expect("Could not read directory") {
        let entry = entry.expect("Could not read directory entry");
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("suite_"))
                == Some(true)
        {
            let current_output_type = process_module_directory(&path, out_dir, parent_output_type);
            process_directory_recursively(&path, out_dir, Some(&current_output_type));
        }
    }
}

// Processes a module directory, generates code, and returns the Output type of the module
fn process_module_directory(
    module_path: &Path,
    out_dir: &str,
    parent_output_type: Option<&str>,
) -> String {
    let module_name = module_path.strip_prefix("src").unwrap().to_str().unwrap();
    let module_name_safe = module_name.replace("/", "_");

    let generated_file_path =
        Path::new(out_dir).join(format!("generated_tests_{}.rs", module_name_safe));
    let mut file =
        File::create(&generated_file_path).expect("Could not create generated test file");

    writeln!(
        file,
        "// Auto-generated code for module `{}`\n",
        module_name
    )
    .unwrap();
    let module_prefix = format!("crate::{}", module_name.replace("/", "::"));

    // Detect the struct name from `mod.rs` in each module
    let main_file_path = module_path.join("mod.rs");
    let struct_name = find_testsuite_struct_in_file(&main_file_path)
        .expect("Expected a struct starting with 'TestSuite' in mod.rs, but none was found");

    // Get list of test files and suites included as `pub mod` in `mod.rs`
    let (test_cases, nested_suites) = partition_modules(&main_file_path);

    // Generate RunnableTrait implementation with async fn run and setup
    writeln!(
        file,
        "impl crate::RunnableTrait for {}::{} {{",
        module_prefix, struct_name
    )
    .unwrap();

    // Determine Input type based on whether it is a root suite or nested suite
    if let Some(output_type) = parent_output_type {
        writeln!(file, "    type Input = {};", output_type).unwrap(); // Nested suite uses parent Output type
    } else {
        writeln!(file, "    type Input = SetupInput;").unwrap(); // Root suite uses `SetupInput`
    }
    writeln!(
        file,
        "    async fn run(input: &Self::Input) -> Result<Self, crate::utils::v7::endpoints::errors::RpcError> {{"
    )
    .unwrap();

    // Call the setup function and store the result in `data`
    writeln!(
        file,
        "        let data = {}::{}::setup(input).await?;",
        module_prefix, struct_name
    )
    .unwrap();

    // Process each declared test module within this suite, passing `data` to each `run`
    for test_name in test_cases {
        writeln!(
            file,
            "        {}::{}::TestCase::run(&data).await?;",
            module_prefix, test_name
        )
        .unwrap();
    }

    // Process each nested suite, dynamically retrieving its struct name and fields
    for nested_suite in nested_suites {
        let nested_module_path = module_path.join(&nested_suite).join("mod.rs");
        let nested_struct_name = find_testsuite_struct_in_file(&nested_module_path)
            .expect("Expected a struct starting with 'TestSuite' in nested suite mod.rs, but none was found");

        // Generate the instantiation code for the nested suite
        writeln!(
            file,
            "        {}::{}::{}::run(&data).await?;",
            module_prefix, nested_suite, nested_struct_name
        )
        .unwrap();
    }

    writeln!(file, "        Ok(data)").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    // Return the current output type for further nested suites
    format!("{}::{}", module_prefix, struct_name)
}

// Partition modules into test cases and nested suites
fn partition_modules(mod_file_path: &Path) -> (Vec<String>, Vec<String>) {
    let content = read_to_string(mod_file_path).expect("Could not read mod.rs file");
    let mut test_cases = Vec::new();
    let mut nested_suites = Vec::new();

    for line in content.lines() {
        if line.trim_start().starts_with("pub mod ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let mod_name = parts[2].trim_end_matches(';').to_string();
                if mod_name.starts_with("suite_") {
                    nested_suites.push(mod_name);
                } else {
                    test_cases.push(mod_name);
                }
            }
        }
    }

    (test_cases, nested_suites)
}

// Utility function to find a struct that starts with "TestSuite" in a specific file, e.g., mod.rs
fn find_testsuite_struct_in_file(file_path: &Path) -> Result<String, String> {
    let content = read_to_string(file_path).map_err(|_| "Could not read file".to_string())?;
    for line in content.lines() {
        if line.starts_with("pub struct TestSuite") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return Ok(parts[2].to_string());
        }
    }
    Err("Expected a struct starting with 'TestSuite' but none was found".to_string())
}
