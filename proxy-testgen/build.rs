use std::fs::{self, File};
use std::io::BufRead;
use std::io::{self, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    // Set the environment variable to rerun the build script if any file in src/state_machines changes
    println!("cargo:rerun-if-changed=src/state_machines");

    // Define the directory to search in
    let state_machines_dir = Path::new("src/state_machines");

    // Define the output file path in the workspace target directory
    let dest_path = Path::new("../proxy/generated_state_machines.rs");
    fs::create_dir_all(dest_path.parent().unwrap())?;
    let mut output = File::create(dest_path)?;

    // Write the necessary imports and start the function definition
    writeln!(output, "pub fn run_generated_state_machines(request_body: String, response_body: String, path: String) {{")?;

    // Traverse the state_machines directory for Rust files
    for entry in fs::read_dir(state_machines_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process Rust files that end with "_state_machine.rs"
        if path.extension().map_or(false, |ext| ext == "rs")
            && path
                .file_name()
                .unwrap()
                .to_str()
                .map_or(false, |name| name.ends_with("_state_machine.rs"))
        {
            // Open the file and read line by line to find an enum ending in "Wrapper"
            let file = File::open(&path)?;
            let reader = io::BufReader::new(file);
            let mut enum_name = None;

            for line in reader.lines() {
                let line = line?;
                // Check for an enum declaration that ends with "Wrapper"
                if line.trim_start().starts_with("pub enum") && line.contains("Wrapper") {
                    if let Some(name) = line.split_whitespace().nth(2) {
                        enum_name = Some(name.to_string());
                        break; // Stop reading the file once we find the enum
                    }
                }
            }

            // Proceed if we found an enum with the "Wrapper" suffix
            if let Some(enum_name) = enum_name {
                let module_name = path.file_stem().unwrap().to_str().unwrap();

                // Generate the fully qualified path to the enum
                let fully_qualified_enum = format!(
                    "proxy_testgen::state_machines::{}::{}",
                    module_name, enum_name
                );

                // Generate the code for each wrapper using the new format
                writeln!(output, "    // Generated code for enum: {}", enum_name)?;
                writeln!(output, "    let result = proxy_testgen::StateMachine::run(")?;
                writeln!(output, "        &mut {}::new(),", fully_qualified_enum)?;
                writeln!(output, "        request_body.clone(),")?;
                writeln!(output, "        response_body.clone(),")?;
                writeln!(output, "        path.clone(),")?;
                writeln!(output, "    );")?;

                writeln!(output, "    match result {{")?;
                writeln!(
                    output,
                    "        proxy_testgen::StateMachineResult::Ok(message) => info!(\"{{}}\", message.green().bold()),"
                )?;
                writeln!(
                    output,
                    "        proxy_testgen::StateMachineResult::Invalid(message) => info!(\"{{}}\", message.red().bold()),"
                )?;
                writeln!(
                    output,
                    "        proxy_testgen::StateMachineResult::Skipped => (),"
                )?;
                writeln!(output, "    }}")?;
            }
        }
    }

    writeln!(output, "}}")?;

    Ok(())
}
