use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevnetError {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("The restart operation failed: {msg}")]
    RestartError { msg: String },
}
