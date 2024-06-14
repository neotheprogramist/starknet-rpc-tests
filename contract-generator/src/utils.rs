use std::{
    fs::File,
    io::{self, Write},
    process::Command,
};

// Function to modify the enum in the contract string
pub fn modify_contract_enum(contract: &str, new_enum: &str) -> String {
    let start_marker = "pub enum BalanceResult {";
    let end_marker = "}\n\n#[starknet::contract]";

    let start_idx =
        contract.find(start_marker).expect("Start marker not found") + start_marker.len();
    let end_idx = contract.find(end_marker).expect("End marker not found");

    let before_enum = &contract[..start_idx];
    let after_enum = &contract[end_idx..];

    format!("{}{}\n{}\n", before_enum, new_enum, after_enum)
}

// Function to write the contract string to a file
pub fn write_contract_to_file(contract: &str, path: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contract.as_bytes())?;
    Ok(())
}

// Function to build the contract using Scarb
pub fn build_contract_with_scarb(package: &str) -> io::Result<()> {
    let output = Command::new("scarb")
        .arg("build")
        .arg("--package")
        .arg(package)
        .output()?;

    if !output.status.success() {
        eprintln!("Scarb build failed: {:?}", output);
        return Err(io::Error::new(io::ErrorKind::Other, "Scarb build failed"));
    }

    println!("Scarb build succeeded!");
    Ok(())
}
