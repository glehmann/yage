use std::io;
use std::{path::PathBuf, result};

use serde_yaml as sy;
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
    YamlError(#[from] sy::Error),
    #[error("can't parse recipient {recipient}: {message}")]
    RecipientParseError { recipient: String, message: String },
    #[error("can't parse key: {message}")]
    KeyParseError { message: String },
    #[error(transparent)]
    DecryptError(#[from] age::DecryptError),
    #[error(transparent)]
    EncryptError(#[from] age::EncryptError),
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("no recipients provided")]
    NoRecipientsError,
    #[error("passphrase not supported")]
    PassphraseUnsupportedError,
    #[error("yaml value is not a map")]
    NotAMapError,
    #[error("yaml value is not a string or a number")]
    NotAStringOrNumberError,
    #[error("invalid file name: {path:?}")]
    InvalidFileNameError { path: PathBuf },
    #[error("editor exited with an error status")]
    EditorError,
    #[error("key not found")]
    KeyNotFoundError,
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
