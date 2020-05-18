use std::collections::{hash_map::DefaultHasher, BTreeSet};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use chrono::{DateTime, Utc};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};

use crate::dictionary::DICTIONARY;
use crate::errors::BlockError;

const BLOCK_WORDSET_LENGTH: usize = 4usize;

#[derive(Debug, Hash, Deserialize, Serialize)]
pub struct BlockBody {
    wordset: BTreeSet<String>,
}

impl BlockBody {
    pub fn new(wordlist: Vec<String>) -> Result<BlockBody, BlockError> {
        let wordset = BTreeSet::from_iter(wordlist.into_iter());

        let body = BlockBody { wordset: wordset };

        body.validate_length().and_then(Self::validate_subset)
    }

    pub fn validate_subset(self) -> Result<BlockBody, BlockError> {
        match self.wordset.is_subset(&DICTIONARY) {
            true => Ok(self),
            false => Err(BlockError::InvalidWordset),
        }
    }

    pub fn validate_length(self) -> Result<BlockBody, BlockError> {
        match self.wordset.iter().len() {
            BLOCK_WORDSET_LENGTH => Ok(self),
            _ => Err(BlockError::InvalidWordsetLength),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    body: BlockBody,
    proposer: String,
    hash: u64,
    parent_hash: u64,
    timestamp: DateTime<Utc>,
}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.body.hash(state);
        self.proposer.hash(state);
        self.parent_hash.hash(state);
        self.timestamp.hash(state);
    }
}

impl Block {
    pub fn new(
        wordlist: Vec<String>,
        proposer: PeerId,
        parent_hash: u64,
    ) -> Result<Block, BlockError> {
        let body = BlockBody::new(wordlist)?;

        let mut block = Block {
            body: body,
            proposer: proposer.to_base58(),
            parent_hash: parent_hash,
            timestamp: Utc::now(),
            hash: 0,
        };

        let mut hasher = DefaultHasher::new();
        block.hash(&mut hasher);
        block.hash = hasher.finish();

        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let wordlist = vec![
            "and".to_string(),
            "for".to_string(),
            "that".to_string(),
            "this".to_string(),
        ];
        let proposer = PeerId::random();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(1337);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer, parent_hash);
        assert!(result.is_ok());

        let _block = result.ok().unwrap();
    }

    #[test]
    fn test_invalid_wordset() {
        let wordlist = vec![
            "and".to_string(),
            "for".to_string(),
            "that".to_string(),
            "notaword".to_string(),
        ];
        let proposer = PeerId::random();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(1337);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer, parent_hash);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), BlockError::InvalidWordset);
    }

    #[test]
    fn test_invalid_wordset_length() {
        let wordlist = vec!["and".to_string(), "for".to_string(), "that".to_string()];
        let proposer = PeerId::random();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(1337);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer, parent_hash);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), BlockError::InvalidWordsetLength);
    }
}
