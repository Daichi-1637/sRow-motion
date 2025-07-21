use std::env::VarError;
use std::io;
use std::path::StripPrefixError;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Env(VarError),
    Path(StripPrefixError),
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<VarError> for AppError {
    fn from(err: VarError) -> Self {
        AppError::Env(err)
    }
}

impl From<StripPrefixError> for AppError {
    fn from(err: StripPrefixError) -> Self {
        AppError::Path(err)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Env(e) => write!(f, "Environment variable error: {}", e),
            AppError::Path(e) => write!(f, "Path error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;
