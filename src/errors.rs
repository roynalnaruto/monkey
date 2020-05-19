use rusty_leveldb::Status;

#[derive(Debug, PartialEq)]
pub enum BlockError {
    UnknownParent,
    KnownBlock,
    InvalidWordset,
    InvalidWordsetLength,
}

#[derive(Debug, PartialEq)]
pub enum StoreError {
    DBOpenError(String),
}

impl From<Status> for StoreError {
    fn from(s: Status) -> StoreError {
        StoreError::DBOpenError(s.err)
    }
}
