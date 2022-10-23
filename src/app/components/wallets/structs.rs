use super::super::accounts::structs::Account;
use crate::crypto::address::get_address;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletsAndLookup {
    pub wallets_bytes: Vec<u8>,
    pub lookup_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub name: String,
    pub mnemonic: String,
    pub seed: [u8; 32],
    pub indexes: Vec<u32>,
    #[serde(skip)]
    pub accounts: Vec<Account>,
    #[serde(skip)]
    pub acc_idx: usize,
}

impl Wallet {
    pub fn new(mnemonic: String, seed: [u8; 32], name: String, prefix: &str) -> Wallet {
        let mut wallet = Wallet {
            name,
            mnemonic,
            seed,
            indexes: vec![0],
            accounts: vec![],
            acc_idx: 0,
        };
        wallet
            .accounts
            .push(Account::with_index(&wallet, 0, prefix));
        wallet
    }
    pub fn new_key(private_key: [u8; 32], name: String, prefix: &str) -> Wallet {
        let public_key = Wallet::get_public_key(&private_key);
        let mut wallet = Wallet {
            name,
            mnemonic: String::from(""),
            seed: [0u8; 32],
            indexes: vec![0],
            accounts: vec![],
            acc_idx: 0,
        };
        wallet.accounts.push(Account {
            index: 0,
            private_key,
            public_key,
            address: get_address(&public_key, Some(prefix)),
            balance: 0,
            receivables: vec![],
            messages: Ok(vec![]),
        });
        wallet
    }

    fn get_public_key(private_key: &[u8; 32]) -> [u8; 32] {
        let dalek = ed25519_dalek::SecretKey::from_bytes(private_key).unwrap();
        let public_key = ed25519_dalek::PublicKey::from(&dalek);
        public_key.to_bytes()
    }
}
