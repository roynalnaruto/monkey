use bincode;
use rusty_leveldb::Status;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownParentBlock,
    DuplicateBlock,
    InvalidProposer(String),
    InvalidSignature,
    InvalidWordset,
    InvalidWordsetLength,

    StdError(String),

    DBError(String),

    SerdeError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::StdError(e.to_string())
    }
}

impl From<Status> for Error {
    fn from(s: Status) -> Error {
        Error::DBError(s.err)
    }
}

type BincodeError = Box<bincode::ErrorKind>;
impl From<BincodeError> for Error {
    fn from(e: BincodeError) -> Error {
        Error::SerdeError(e.to_string())
    }
}
