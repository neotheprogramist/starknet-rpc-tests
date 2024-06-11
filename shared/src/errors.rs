use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum RequestOrParseError {
    Reqwest(reqwest::Error),
    Url(url::ParseError),
}

impl fmt::Display for RequestOrParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestOrParseError::Reqwest(e) => write!(f, "{}", e),
            RequestOrParseError::Url(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for RequestOrParseError {}

impl From<reqwest::Error> for RequestOrParseError {
    fn from(err: reqwest::Error) -> RequestOrParseError {
        RequestOrParseError::Reqwest(err)
    }
}

impl From<url::ParseError> for RequestOrParseError {
    fn from(err: url::ParseError) -> RequestOrParseError {
        RequestOrParseError::Url(err)
    }
}
