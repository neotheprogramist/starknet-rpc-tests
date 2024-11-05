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

    // Iterate over each subdirectory in `src`
    process_directory_recursively(src_dir, &out_dir);

    println!("cargo:rerun-if-changed=src");
}

// Rekurencyjnie przetwarza katalogi z prefiksem "suite_"
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
    let module_name_safe = module_name.replace("/", "_"); // Zamień na bezpieczną nazwę

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

    // Find the `struct` and `trait` names from the main file in this module
    let main_file_path = module_path.join("mod.rs");
    let main_content = read_to_string(&main_file_path).expect("Could not read mod.rs file");

    let struct_name = find_struct_name(&main_content).unwrap_or("TestSuite".to_string());
    let trait_name = find_trait_name(&main_content).unwrap_or("TestSuiteCommon".to_string());

    // Implement the detected trait for the detected struct
    writeln!(
        file,
        "impl {}::{} for {}::{} {{",
        module_prefix, trait_name, module_prefix, struct_name
    )
    .unwrap();
    writeln!(file, "    fn run(&self) {{").unwrap();

    // Look for test files in the module directory
    for entry in fs::read_dir(module_path).expect("Could not read module directory") {
        let entry = entry.expect("Could not read directory entry");
        let path = entry.path();
        if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
            if file_name.starts_with("test")
                && path.extension().and_then(|s| s.to_str()) == Some("rs")
            {
                let content = read_to_string(&path).expect("Could not read test file");
                let test_struct_name = find_struct_name(&content).unwrap_or("TestCase".to_string());
                let test_trait_name =
                    find_trait_name(&content).unwrap_or("TestCaseTrait".to_string());

                writeln!(file, "        {{").unwrap();
                writeln!(
                    file,
                    "            use {}::{}::{};",
                    module_prefix, file_name, test_trait_name
                )
                .unwrap();
                writeln!(
                    file,
                    "            let test_case = {}::{}::{} {{ tmp: String::from(\"value\") }};",
                    module_prefix, file_name, test_struct_name
                )
                .unwrap();
                writeln!(file, "            test_case.run();").unwrap();
                writeln!(file, "        }}").unwrap();
            }
        }
    }

    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}

fn find_struct_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("pub struct ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return Some(parts[2].to_string());
        }
    }
    None
}

fn find_trait_name(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("pub trait ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return Some(parts[2].to_string());
        }
    }
    None
}
