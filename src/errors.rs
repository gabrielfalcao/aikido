use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    IOErr(std::io::Error),
    CommandExecutionErr((i32, String)),
    UTF8DecodeErr(FromUtf8Error),
    UnknownErr(UnknownError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IOErr(e) => write!(f, "IOError: {}", e),
            Error::UTF8DecodeErr(e) => write!(f, "UTF8DecodeError: {}", e),
            Error::CommandExecutionErr((exit_code, message)) => write!(f, "CommandExecutionError {}: {}", exit_code, message),
            Error::UnknownErr(e) => write!(f, "UnknownError: {:#?}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(or: std::io::Error) -> Self {
        Error::IOErr(or)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(or: FromUtf8Error) -> Self {
        Error::UTF8DecodeErr(or)
    }
}


impl From<(i32, String)> for Error {
    fn from(e: (i32, String)) -> Self {
        Error::CommandExecutionErr(e)
    }
}

impl From<UnknownError> for Error {
    fn from(e: UnknownError) -> Self {
        Error::UnknownErr(e)
    }
}

#[derive(Debug)]
pub struct UnknownError {
    reason: String,
}
impl std::fmt::Display for UnknownError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl UnknownError {
    pub fn new(reason: String) -> UnknownError {
        UnknownError { reason }
    }
}

pub enum Result<T> {
    Ok(T),
    Err(Error),
}
