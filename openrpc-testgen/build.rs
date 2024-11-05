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
    let struct_name = match find_struct_name_in_file(&main_file_path) {
        Some(name) => name,
        None => "TestSuite".to_string(), // default if no struct found
    };

    // Get list of test files included as `pub mod` in `mod.rs`
    let declared_tests = find_pub_mods_in_mod(&main_file_path);

    // Implement `RunnableTrait` for the detected struct
    writeln!(
        file,
        "impl crate::RunnableTrait for {}::{} {{",
        module_prefix, struct_name
    )
    .unwrap();
    writeln!(file, "    fn run(&self) {{").unwrap();

    // Process test files that are declared as `pub mod` within `mod.rs`
    for entry in fs::read_dir(module_path).expect("Could not read module directory") {
        let entry = entry.expect("Could not read directory entry");
        let path = entry.path();
        if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
            if declared_tests.contains(&file_name.to_string())
                && path.extension().and_then(|s| s.to_str()) == Some("rs")
            {
                let content = read_to_string(&path).expect("Could not read test file");
                let test_struct_name = find_struct_name(&content).unwrap_or("TestCase".to_string());

                writeln!(
                    file,
                    "        let test_case = {}::{}::{} {{ tmp: String::from(\"value\") }};",
                    module_prefix, file_name, test_struct_name
                )
                .unwrap();
                writeln!(file, "        test_case.run();").unwrap();
            }
        }
    }

    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}

// Finds the list of `pub mod` declarations in `mod.rs`
fn find_pub_mods_in_mod(mod_file_path: &Path) -> Vec<String> {
    let content = read_to_string(mod_file_path).expect("Could not read mod.rs file");
    let mut pub_mods = Vec::new();

    for line in content.lines() {
        if line.trim_start().starts_with("pub mod ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let mod_name = parts[2].trim_end_matches(';').to_string();
                pub_mods.push(mod_name);
            }
        }
    }

    pub_mods
}

// Utility function to find the struct name in a specific file, e.g., mod.rs
fn find_struct_name_in_file(file_path: &Path) -> Option<String> {
    let content = read_to_string(file_path).expect("Could not read file");
    find_struct_name(&content)
}

// Utility functions for detecting struct and trait names
fn find_struct_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("pub struct ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return Some(parts[2].to_string());
        }
    }
    None
}
