use bip39::{Mnemonic, MnemonicType, Language, Seed};
use bitcoin::util::bip32::ExtendedPrivKey;

pub static mut BLOCKCHAIN_ADDRESS: String = String::new();

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
            println_debug!("{:#?}", e);
            std::process::exit(1);
        }
    }
}

pub fn init() {
    // Compressed address legacy (P2PKH)
    let mnemonic: Mnemonic;
    let metadata = std::fs::metadata(super::config::SEED_PHRASE_PATH.to_string());
    match metadata {
        Ok(_) => {
            mnemonic = Mnemonic::from_phrase(&read_seed_phrase(), Language::English).unwrap();
        },
        Err(_) => {
            mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
            write_seed_phrase(mnemonic.phrase());
        },
    }

    let seed = Seed::new(&mnemonic, "");
    let seed_bytes: &[u8] = seed.as_bytes();

    let master_key = ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, seed_bytes).unwrap();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let public_key = bitcoin::PublicKey::from_private_key(&secp, &master_key.to_priv());
    let address = bitcoin::Address::p2pkh(&public_key, bitcoin::Network::Bitcoin);

    unsafe {BLOCKCHAIN_ADDRESS = address.to_string();}

}