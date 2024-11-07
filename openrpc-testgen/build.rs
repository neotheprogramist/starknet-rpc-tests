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

    // Process each suite_ directory in `src`
    process_directory_recursively(src_dir, &out_dir);

    println!("cargo:rerun-if-changed=src");
}

// Recursively processes directories with prefix "suite_"
fn process_directory_recursively(dir: &Path, out_dir: &str) {
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
            process_module_directory(&path, out_dir);
            process_directory_recursively(&path, out_dir);
        }
    }
}

fn process_module_directory(module_path: &Path, out_dir: &str) {
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
    let struct_name = match find_testsuite_struct_in_file(&main_file_path) {
        Some(name) => name,
        None => "TestSuite".to_string(), // default if no struct found
    };

    // Get list of test files and suites included as `pub mod` in `mod.rs`
    let (test_cases, nested_suites) = partition_modules(&main_file_path);

    // Generate RunnableTrait implementation with async fn run and setup
    writeln!(
        file,
        "impl crate::RunnableTrait for {}::{} {{",
        module_prefix, struct_name
    )
    .unwrap();
    writeln!(file, "    type Output = ();").unwrap();
    writeln!(
        file,
        "    async fn run(&self) -> Result<Self::Output, crate::utils::v7::endpoints::errors::RpcError> {{"
    )
    .unwrap();

    // Call the setup function
    writeln!(file, "        let data = self.setup().await?;").unwrap();

    // Process each declared test module within this suite
    for test_name in test_cases {
        writeln!(
            file,
            "        let test_case = {}::{}::TestCase {{ data: data.clone() }};",
            module_prefix, test_name
        )
        .unwrap();
        writeln!(file, "        test_case.run().await?;").unwrap();
    }

    // Process each nested suite, dynamically retrieving its struct name and fields
    for nested_suite in nested_suites {
        let nested_module_path = module_path.join(&nested_suite).join("mod.rs");
        if let Some(nested_struct_name) = find_testsuite_struct_in_file(&nested_module_path) {
            let fields = get_struct_fields(&nested_module_path);

            // Generate the instantiation code for the nested suite with required fields
            writeln!(
                file,
                "        let nested_suite = {}::{}::{} {{",
                module_prefix, nested_suite, nested_struct_name
            )
            .unwrap();

            for field in fields {
                writeln!(file, "            {}: data.{0}.clone(),", field).unwrap();
            }

            writeln!(file, "        }};").unwrap();
            writeln!(file, "        nested_suite.run().await?;").unwrap();
        }
    }

    writeln!(file, "        Ok(())").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
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
fn find_testsuite_struct_in_file(file_path: &Path) -> Option<String> {
    let content = read_to_string(file_path).expect("Could not read file");
    for line in content.lines() {
        if line.starts_with("pub struct TestSuite") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return Some(parts[2].to_string());
        }
    }
    None
}

// Utility function to extract struct fields from a module's `mod.rs` file
fn get_struct_fields(mod_file_path: &Path) -> Vec<String> {
    let content = read_to_string(mod_file_path).expect("Could not read mod.rs file");
    let mut fields = Vec::new();
    let mut inside_struct = false;

    for line in content.lines() {
        if line.starts_with("pub struct TestSuite") {
            inside_struct = true;
        } else if inside_struct && line.starts_with("}") {
            break;
        } else if inside_struct {
            // Remove "pub" and get the field name before the colon
            if let Some(field_name) = line
                .replace("pub ", "")
                .split(':')
                .next()
                .map(|s| s.trim().to_string())
            {
                fields.push(field_name);
            }
        }
    }

    fields
}
