use crate::block_build::errors::Error;
use pathfinder_types::types::block_builder_input::B11rInput;
use serde::Serialize;
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io::Read,
};
use tracing::{error, info};

pub fn read_input_file(input_path: PathBuf) -> Result<B11rInput, Error> {
    let mut input_file = File::open(input_path)?;

    let mut input = String::new();
    input_file.read_to_string(&mut input)?;
    let input_data: B11rInput = serde_json::from_str(&input)?;
    Ok(input_data)
}

pub fn write_block_file<T: Serialize>(file_path: PathBuf, data: &T) -> Result<(), Error> {
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(&file_path)?;
    serde_json::to_writer_pretty(&file, data).map_err(|e| {
        error!("Failed to write JSON to file: {}", e);
        e
    })?;

    info!("Block written into {}", file_path.display());
    Ok(())
}
