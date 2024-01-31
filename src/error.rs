use std::{path::PathBuf, result};

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{path}: {source}")]
    PathIoError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error("can't parse recipient {recipient}: {message}")]
    RecipientParseError { recipient: String, message: String },
    #[error(transparent)]
    EncryptError(#[from] age::EncryptError),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("no recipients provided")]
    NoRecipientsError,
}

/// Alias for a `Result` with the error type `AppError`.
pub type Result<T> = result::Result<T, AppError>;

pub trait IOResultExt<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T>;
}

impl<T> IOResultExt<T> for io::Result<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T> {
        self.map_err(|source| AppError::PathIoError {
            source,
            path: path.into(),
        })
    }
}
