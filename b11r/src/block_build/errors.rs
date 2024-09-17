use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error when reading file {path}")]
    ReadFileError {
        source: std::io::Error,
        path: String,
    },
    #[error("The file does not exist")]
    FileNotFound,

}