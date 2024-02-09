use std::io;
use std::{path::PathBuf, result};

use serde_yaml as sy;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YageError {
    #[error("{path}: {source}")]
    PathIo {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Yaml(#[from] sy::Error),
    #[error("can't parse recipient {recipient}: {message}")]
    RecipientParse { recipient: String, message: String },
    #[error("can't parse key: {message}")]
    KeyParse { message: String },
    #[error(transparent)]
    Decrypt(#[from] age::DecryptError),
    #[error(transparent)]
    Encrypt(#[from] age::EncryptError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),
    #[error("no recipients provided")]
    NoRecipients,
    #[error("passphrase not supported")]
    PassphraseUnsupported,
    #[error("yaml value is not a map")]
    NotAMap,
    #[error("yaml value is not a string or a number")]
    NotAStringOrNumber,
    #[error("invalid file name: {path:?}")]
    InvalidFileName { path: PathBuf },
    #[error("editor exited with an error status")]
    Editor,
    #[error("key not found")]
    KeyNotFound,
    #[error("stdin can't be modified in place")]
    InPlaceStdin,
}

/// Alias for a `Result` with the error type `AppError`.
pub type Result<T> = result::Result<T, YageError>;

pub trait IOResultExt<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T>;
}

impl<T> IOResultExt<T> for io::Result<T> {
    fn path_ctx<P: Into<PathBuf>>(self, path: P) -> Result<T> {
        self.map_err(|source| YageError::PathIo {
            source,
            path: path.into(),
        })
    }
}
