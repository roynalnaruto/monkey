use libp2p::identity::error as IdentityError;
use rusty_leveldb::Status;

#[derive(Debug, PartialEq)]
pub enum BlockError {
    StoreError(StoreError),
    UnknownParentBlock,
    DuplicateBlock,
    InvalidProposer(String),
    InvalidSignature,
    InvalidWordset,
    InvalidWordsetLength,
}

impl From<IdentityError::DecodingError> for BlockError {
    fn from(e: IdentityError::DecodingError) -> BlockError {
        BlockError::InvalidProposer(e.to_string())
    }
}

impl From<StoreError> for BlockError {
    fn from(e: StoreError) -> BlockError {
        BlockError::StoreError(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum StoreError {
    DBError(String),
}

impl From<Status> for StoreError {
    fn from(s: Status) -> StoreError {
        StoreError::DBError(s.err)
    }
}
