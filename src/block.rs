use std::collections::{hash_map::DefaultHasher, BTreeSet};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use bincode::serialize;
use chrono::{DateTime, Utc};
use libp2p::identity::ed25519::{Keypair, PublicKey};
use serde::{Deserialize, Serialize};

use crate::dictionary::DICTIONARY;
use crate::errors::BlockError;

const BLOCK_WORDSET_LENGTH: usize = 4usize;

#[derive(Clone, Debug, Hash, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    body: BlockBody,
    proposer: [u8; 32],
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
        proposer: PublicKey,
        parent_hash: u64,
    ) -> Result<Block, BlockError> {
        let body = BlockBody::new(wordlist)?;

        let mut block = Block {
            body: body,
            proposer: proposer.encode(),
            parent_hash: parent_hash,
            timestamp: Utc::now(),
            hash: 0,
        };

        let mut hasher = DefaultHasher::new();
        block.hash(&mut hasher);
        block.hash = hasher.finish();

        Ok(block)
    }

    pub fn sign(self, keypair: &Keypair) -> SignedBlock {
        let message = serialize(&self).unwrap();

        let signature = keypair.sign(&message);

        SignedBlock {
            message: self,
            signature: signature,
        }
    }
}

type Signature = Vec<u8>;

#[derive(Debug)]
pub struct SignedBlock {
    message: Block,
    signature: Signature,
}

impl SignedBlock {
    pub fn verify_signature(&self, public_key: &PublicKey) -> bool {
        let message = serialize(&self.message).unwrap();

        public_key.verify(&message, &self.signature)
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
        let proposer = Keypair::generate();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(1337);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer.public(), parent_hash);

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_wordset() {
        let wordlist = vec![
            "and".to_string(),
            "for".to_string(),
            "that".to_string(),
            "notaword".to_string(),
        ];
        let proposer = Keypair::generate();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(1337);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer.public(), parent_hash);

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), BlockError::InvalidWordset);
    }

    #[test]
    fn test_invalid_wordset_length() {
        let wordlist = vec!["and".to_string(), "for".to_string(), "that".to_string()];
        let proposer = Keypair::generate();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(1337);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer.public(), parent_hash);

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), BlockError::InvalidWordsetLength);
    }

    #[test]
    fn test_sign_verify_block() {
        let wordlist = vec![
            "and".to_string(),
            "for".to_string(),
            "that".to_string(),
            "this".to_string(),
        ];
        let proposer = Keypair::generate();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(1337);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer.public(), parent_hash);
        let block = result.ok().unwrap();
        let signed_block = block.sign(&proposer);

        let not_proposer = Keypair::generate();
        assert!(signed_block.verify_signature(&proposer.public()));
        assert_eq!(signed_block.verify_signature(&not_proposer.public()), false);
    }
}
