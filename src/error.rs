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
