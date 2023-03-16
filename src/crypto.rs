use std::str::FromStr;

use bip39::{Mnemonic, MnemonicType, Language, Seed};
use bitcoin::{util::bip32::ExtendedPrivKey, secp256k1::{Message, ecdsa}, hashes::sha256};
use hex;

pub static mut BLOCKCHAIN_ADDRESS: String = String::new();
pub static mut PUBLIC_KEY: Option<bitcoin::secp256k1::PublicKey> = None;
static mut PRIVATE_KEY: Option<bitcoin::secp256k1::SecretKey> = None;

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
    // derivationpath "m/"
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

    // generate Blockchain address
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let public_key = bitcoin::PublicKey::from_private_key(&secp, &master_key.to_priv());
    let address = bitcoin::Address::p2pkh(&public_key, bitcoin::Network::Bitcoin);

    let keypair = master_key.to_keypair(&secp);
    let private_key = keypair.secret_key();

    unsafe {
        BLOCKCHAIN_ADDRESS = address.to_string();
        PUBLIC_KEY = Some(keypair.public_key());
        PRIVATE_KEY = Some(private_key);
        println_debug!("{:#?}\n{:#?}", hex::encode(private_key.secret_bytes()), PUBLIC_KEY.unwrap());
    }

}

pub fn str_to_msg(message: &str) -> secp256k1::Message {
    let scep_message: secp256k1::Message = secp256k1::Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    return scep_message;
}

pub fn sign_string(message: &str) -> String {
    let scep_message: bitcoin::secp256k1::Message = bitcoin::secp256k1::Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    let private_key = unsafe{PRIVATE_KEY.unwrap()};
    let scep_context = bitcoin::secp256k1::Secp256k1::signing_only();

    let signature = scep_context.sign_ecdsa(&scep_message, &private_key);

    return signature.to_string();
}

pub fn verify_signature(signature: &str, message: &str, public_key: &str) -> bool {
    let result = || -> Result<bool, secp256k1::Error> {
        let message_secp = str_to_msg(message);
        let scep_signaure = ecdsa::Signature::from_str(signature)?;
        let secp_public_key = secp256k1::PublicKey::from_str(public_key)?;
        
        match scep_signaure.verify(&message_secp, &secp_public_key) {
            Ok(_) => {return Ok(true);},
            Err(e) => {
                println_debug!("{:#?}", e);
                return Ok(false);
            }
        }
    }();

    match result {
        Ok(_) => {return result.unwrap();},
        Err(e) => {
            println_debug!("TEST");
            return false;
        }
    }
}