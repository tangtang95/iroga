use std::path::PathBuf;

use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    StripPrefix(#[from] ::std::path::StripPrefixError),
    #[error("{0} is not a directory")]
    NotDir(PathBuf),
    #[error("output path already exists: {0}")]
    OutputPathExists(PathBuf),
    #[error("{0} has invalid unicode")]
    InvalidUnicode(PathBuf),
    #[error("could not find default name from {0}")]
    CannotDetectDefaultName(PathBuf),
    #[error("parsing error due to invalid iro flags {0}")]
    InvalidIroFlags(i32),
    #[error("parsing error due to invalid iro version {0}")]
    InvalidIroVersion(i32),
    #[error("failed to parse binary data")]
    CannotParseBinary(nom::Err<::nom::error::Error<Vec<u8>>>),
    #[error("parsing error due to invalid file flags {0}")]
    InvalidFileFlags(i32),
    #[error("invalid utf16 {0}")]
    InvalidUtf16(String),
    #[error("parent file path does not exists: {0}")]
    ParentPathDoesNotExist(PathBuf),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::CannotParseBinary(err.map_input(|input| input.into()))
    }
}
