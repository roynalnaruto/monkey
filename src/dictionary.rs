use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

lazy_static! {
    pub static ref DICTIONARY: BTreeSet<String> = {
        let mut d = BTreeSet::<String>::new();

        let p = Path::new(".data").join("dictionary.txt");
        let f = File::open(&p).unwrap();
        let wordlist = BufReader::new(&f);
        for word in wordlist.lines() {
            let w = word.unwrap();
            d.insert(w);
        }

        d
    };
}

#[test]
fn dict() {
    assert_eq!(DICTIONARY.iter().len(), 9894usize);

    assert!(DICTIONARY.get(&String::from("word")).is_some());
    assert!(DICTIONARY.get(&String::from("notaword")).is_none());

    assert!(DICTIONARY.get(&String::from("repor")).is_none());
    assert!(DICTIONARY.get(&String::from("report")).is_some());
    assert!(DICTIONARY.get(&String::from("reportr")).is_none());
}
