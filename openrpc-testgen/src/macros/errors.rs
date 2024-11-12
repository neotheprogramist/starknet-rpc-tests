use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssertionNoPanicError {
    #[error("Assertion failed: {0}")]
    AssertionNoPanicFailed(String),
}
