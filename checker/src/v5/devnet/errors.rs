use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevnetError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error("The restart operation failed: {msg}")]
    Restart { msg: String },
}
