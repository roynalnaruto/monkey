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
