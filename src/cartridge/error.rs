use std::{
    error::Error,
    fmt::{Debug, Display},
    io::Error as IoError,
};

#[derive(Debug)]
pub enum CartridgeError {
    InvalidHeader,
    UnsupportedVersion,
    PALNotSupported,
    UnsupportedFile,
    FileTooLarge(u64),
    IOError(IoError),
}

impl Error for CartridgeError {}

impl Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self::Debug::fmt(&self, f)
    }
}

impl From<IoError> for CartridgeError {
    fn from(value: IoError) -> Self {
        Self::IOError(value)
    }
}
