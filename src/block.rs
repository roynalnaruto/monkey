use std::collections::{hash_map::DefaultHasher, BTreeSet};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use bincode::serialize;
use chrono::{DateTime, Utc};
use libp2p::identity::ed25519::{Keypair, PublicKey};
use serde::{Deserialize, Serialize};

use crate::dictionary::DICTIONARY;
use crate::errors::Error;

const BLOCK_WORDSET_LENGTH: usize = 4usize;
const GENESIS_BLOCK_DATA: u16 = 1337;

lazy_static! {
    static ref GENESIS_BLOCK_HASH: u64 = {
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(GENESIS_BLOCK_DATA);

        hasher.finish()
    };
}

#[derive(Clone, Debug, Hash, Deserialize, Serialize)]
pub struct BlockBody {
    wordset: BTreeSet<String>,
}

impl BlockBody {
    pub fn new(wordlist: Vec<String>) -> Result<BlockBody, Error> {
        let wordset = BTreeSet::from_iter(wordlist.into_iter());

        let body = BlockBody { wordset: wordset };

        body.validate_length().and_then(Self::validate_subset)
    }

    #[allow(dead_code)]
    pub fn validate(self) -> Result<Self, Error> {
        self.validate_length().and_then(Self::validate_subset)
    }

    pub fn validate_subset(self) -> Result<BlockBody, Error> {
        match self.wordset.is_subset(&DICTIONARY) {
            true => Ok(self),
            false => Err(Error::InvalidWordset),
        }
    }

    pub fn validate_length(self) -> Result<BlockBody, Error> {
        match self.wordset.iter().len() {
            BLOCK_WORDSET_LENGTH => Ok(self),
            _ => Err(Error::InvalidWordsetLength),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    body: BlockBody,
    proposer: [u8; 32],
    pub hash: u64,
    pub parent_hash: u64,
    timestamp: DateTime<Utc>,
}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.body.hash(state);
    }
}

impl Block {
    pub fn new(
        wordlist: Vec<String>,
        proposer: PublicKey,
        parent_hash: u64,
    ) -> Result<Block, Error> {
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

    pub fn validate(self) -> Result<Self, Error> {
        self.body
            .clone()
            .validate_length()
            .and_then(BlockBody::validate_subset)
            .map(|_| self)
    }

    pub fn sign(self, keypair: &Keypair) -> SignedBlock {
        let message = serialize(&self).unwrap();

        let signature = keypair.sign(&message);

        SignedBlock {
            message: self,
            signature: signature,
        }
    }

    pub fn genesis_block() -> (u64, Vec<u8>, Vec<u8>) {
        (
            *GENESIS_BLOCK_HASH,
            GENESIS_BLOCK_HASH.to_be_bytes().to_vec(),
            GENESIS_BLOCK_DATA.to_be_bytes().to_vec(),
        )
    }
}

type Signature = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedBlock {
    pub message: Block,
    signature: Signature,
}

impl SignedBlock {
    pub fn verify_signature(&self) -> bool {
        match PublicKey::decode(&self.message.proposer) {
            Ok(public_key) => {
                let message = serialize(&self.message).unwrap();
                public_key.verify(&message, &self.signature)
            }
            _ => false,
        }
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
        assert_eq!(result.err().unwrap(), Error::InvalidWordset);
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
        assert_eq!(result.err().unwrap(), Error::InvalidWordsetLength);
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
        let signed_block = block.clone().sign(&proposer);

        assert!(signed_block.verify_signature());

        let invalid_signed_block = SignedBlock {
            message: block,
            signature: vec![1, 2, 3, 4],
        };
        assert_eq!(invalid_signed_block.verify_signature(), false)
    }
}
