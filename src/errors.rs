#[derive(Debug, PartialEq)]
pub enum BlockError {
    UnknownParent,
    KnownBlock,
    InvalidWordset,
    InvalidWordsetLength,
}
