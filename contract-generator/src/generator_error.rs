use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Scarb build failed")]
    ScarbBuildFailed,
}
