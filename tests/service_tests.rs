#![feature(custom_test_frameworks)]
#![test_runner(crate::my_runner)]
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::Hasher;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;

use libp2p::identity::ed25519::Keypair;
use monkeylib::{Block, Error, Service};

#[cfg(test)]
fn my_runner(tests: &[&dyn Fn(&Service)]) {
    let path = Path::new(".data").join(".test").join("servicedata");
    let service = build_service(&path);

    for test in tests {
        match panic::catch_unwind(AssertUnwindSafe(|| {
            test(&service);
        })) {
            Err(e) => {
                fs::remove_dir_all(&path).ok().unwrap();
                panic!("Error: {:?}", e);
            }
            Ok(..) => {
                println!("passed");
            }
        }
    }

    fs::remove_dir_all(&path).ok().unwrap();
}

fn build_service(path: &Path) -> Service {
    let service = Service::new(&path).ok().unwrap();

    service.import_genesis().ok().unwrap();

    service
}

#[test_case]
fn test_import_block(service: &Service) {
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

    assert!(
        service.import_block(&signed_block).is_ok(),
        "import_block not ok"
    );
}

#[test_case]
fn test_import_block_invalid_parent(service: &Service) {
    let wordlist = vec![
        "and".to_string(),
        "for".to_string(),
        "old".to_string(),
        "new".to_string(),
    ];
    let proposer = Keypair::generate();
    let mut hasher = DefaultHasher::new();
    hasher.write_u16(7331);
    let parent_hash = hasher.finish();

    let result = Block::new(wordlist, proposer.public(), parent_hash);
    let block = result.ok().unwrap();
    let signed_block = block.clone().sign(&proposer);

    let result = service.import_block(&signed_block);

    assert!(result.is_err(), "import_block expected failure but was ok");
    assert_eq!(
        result.err().unwrap(),
        Error::UnknownParentBlock,
        "import_block error mismatch"
    );
}

#[test_case]
fn test_import_duplicate_block(service: &Service) {
    let wordlist = vec![
        "never".to_string(),
        "ever".to_string(),
        "where".to_string(),
        "when".to_string(),
    ];
    let proposer = Keypair::generate();
    let mut hasher = DefaultHasher::new();
    hasher.write_u16(1337);
    let parent_hash = hasher.finish();

    let result = Block::new(wordlist, proposer.public(), parent_hash);
    let block = result.ok().unwrap();
    let signed_block = block.clone().sign(&proposer);

    service.import_block(&signed_block).unwrap();

    assert!(
        service.import_block(&signed_block).is_err(),
        "import_block expected failure but was ok"
    );
}
