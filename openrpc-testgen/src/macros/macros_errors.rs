use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum AssertionNoPanicError {
    #[error("Assertion failed: {0}")]
    AssertionNoPanicFailed(String),
}
