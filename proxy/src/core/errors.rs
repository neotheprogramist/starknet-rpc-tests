use openssl::error::ErrorStack;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    ReqwestErrot(#[from] reqwest::Error),
    #[error("Unsupported HTTP method: {method}")]
    MethodError { method: String },
    #[error(transparent)]
    KeyError(#[from] ErrorStack),
    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseUrlError(#[from] url::ParseError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error)
    
}
