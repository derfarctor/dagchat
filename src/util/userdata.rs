use super::coin::Coin;
use super::wallet::Wallet;
use std::collections::HashMap;

pub struct UserData {
    pub password: String,
    pub wallets: Vec<Wallet>,
    pub wallet_idx: usize,
    pub lookup: HashMap<String, String>,
    pub coin: Coin,
    pub encrypted_bytes: Vec<u8>,
}

impl UserData {
    pub fn new() -> Self {
        UserData {
            password: String::from(""),
            wallets: vec![],
            wallet_idx: 0,
            lookup: HashMap::new(),
            coin: Coin::nano(),
            encrypted_bytes: vec![],
        }
    }
}
