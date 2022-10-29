use super::coin::Coin;
use super::components::wallets::structs::Wallet;
use arboard::Clipboard;
use std::collections::HashMap;

pub struct UserData {
    pub password: String,
    pub clipboard: Clipboard,
    pub wallets: Vec<Wallet>,
    pub wallet_idx: usize,
    pub lookup: HashMap<String, String>,
    pub addressbook: HashMap<String, String>,
    pub coin: Coin,
    pub encrypted_bytes: Vec<u8>,
}

impl UserData {
    pub fn new() -> Self {
        UserData {
            password: String::from(""),
            clipboard: Clipboard::new().unwrap(),
            wallets: vec![],
            wallet_idx: 0,
            lookup: HashMap::new(),
            addressbook: HashMap::new(),
            coin: Coin::nano(),
            encrypted_bytes: vec![],
        }
    }
}
