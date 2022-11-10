use crate::app::components::{
    messages::structs::SavedMessage, receive::structs::Receivable, wallets::structs::Wallet,
};
use crate::crypto::address::get_address;
use crate::crypto::keys::get_private_key;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub index: u32,
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub address: String,
    pub balance: u128,
    pub receivables: Vec<Receivable>,
    pub messages: Result<Vec<SavedMessage>, String>,
}

impl Account {
    pub fn with_index(wallet: &Wallet, index: u32, prefix: &str) -> Account {
        if wallet.mnemonic.is_empty() {
            // Generate using seed as private key.
            let public_key = Wallet::get_public_key(&wallet.seed);
            Account {
                index,
                private_key: wallet.seed,
                public_key,
                address: get_address(&public_key, Some(prefix)),
                balance: 0,
                receivables: vec![],
                messages: Ok(vec![]),
            }
        } else {
            let (private_key, public_key) = Account::get_keypair(&wallet.seed, index);
            Account {
                index,
                private_key,
                public_key,
                address: get_address(&public_key, Some(prefix)),
                balance: 0,
                receivables: vec![],
                messages: Ok(vec![]),
            }
        }
    }

    pub fn get_keypair(seed: &[u8; 32], index: u32) -> ([u8; 32], [u8; 32]) {
        let private_key = get_private_key(seed, index);
        let public_key = Account::get_public_key(&private_key);
        (private_key, public_key)
    }
    pub fn get_public_key(private_key: &[u8; 32]) -> [u8; 32] {
        let dalek = ed25519_dalek::SecretKey::from_bytes(private_key).unwrap();
        let public_key = ed25519_dalek::PublicKey::from(&dalek);
        public_key.to_bytes()
    }
}
