use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevnetError {
    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}
