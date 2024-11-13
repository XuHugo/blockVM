

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),
    #[error("Error accessing {0}: {1}")]
    IO(String, #[source] std::io::Error),
    #[error("Error (de)serializing {0}: {1}")]
    BCS(&'static str, #[source] bcs::Error),
    #[error("Error (de)serializing {0}: {1}")]
    Yaml(String, #[source] serde_yaml::Error),
    #[error("Config is missing expected value: {0}")]
    Missing(&'static str),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}


#[derive(Debug, Error)]
pub enum RestError {
    #[error("BCS ser/de error {0}")]
    Bcs(bcs::Error),
    #[error("Timeout waiting for transaction {0}")]
    Timeout(&'static str),
    #[error("Unknown error {0}")]
    Unknown(anyhow::Error),
}


impl From<bcs::Error> for RestError {
    fn from(err: bcs::Error) -> Self {
        Self::Bcs(err)
    }
}


impl From<anyhow::Error> for RestError {
    fn from(err: anyhow::Error) -> Self {
        Self::Unknown(err)
    }
}





#[derive(Debug, Error, PartialEq, Eq)]
pub enum GitError {
    #[error("Http error, status code: {0}, status text: {1}, body: {2}")]
    HttpError(u16, String, String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Missing field {0}")]
    MissingField(String),
    #[error("404: Not Found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<std::io::Error> for GitError {
    fn from(error: std::io::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<ureq::Response> for GitError {
    fn from(resp: ureq::Response) -> Self {
        if let Some(e) = resp.synthetic_error() {
            // Local error
            GitError::InternalError(e.to_string())
        } else {
            // Clear the buffer
            let status = resp.status();
            let status_text = resp.status_text().to_string();
            match resp.into_string() {
                Ok(body) => GitError::HttpError(status, status_text, body),
                Err(e) => GitError::InternalError(e.to_string()),
            }
        }
    }
}

impl From<serde_json::Error> for GitError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}


