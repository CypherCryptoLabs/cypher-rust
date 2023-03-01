use bip39::{Mnemonic, MnemonicType, Language, Seed};
use std::fs;

fn read_seed_phrase() -> String {
    let mut file = std::fs::File::open(super::config::SEED_PHRASE_PATH.to_string()).unwrap();
    let mut contents = String::new();
    std::io::Read::read_to_string(&mut file, &mut contents).unwrap();

    return contents
}

fn write_seed_phrase(phrase: &str) {
    let mut file = std::fs::File::create(super::config::SEED_PHRASE_PATH.to_string()).unwrap();
    let resutlt = std::io::Write::write_all(&mut file, phrase.to_string().as_bytes());
    
    match resutlt {
        Ok(_) => return,
        Err(e) => {
            println!("{:#?}", e);
            std::process::exit(1);
        }
    }
}

pub fn init() {
    let mnemonic: Mnemonic;
    let phrase: &str;
    let metadata = fs::metadata(super::config::SEED_PHRASE_PATH.to_string());
    match metadata {
        Ok(_) => {
            mnemonic = Mnemonic::from_phrase(&read_seed_phrase(), Language::English).unwrap();
            phrase = mnemonic.phrase();
        },
        Err(_) => {
            mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
            phrase = mnemonic.phrase();
            write_seed_phrase(phrase);
        },
    }
    
    println!("phrase: {}", phrase);

    let seed = Seed::new(&mnemonic, "");
    let seed_bytes: &[u8] = seed.as_bytes();
    println!("{:X}", seed);
}