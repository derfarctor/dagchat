use super::coin::*;
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
    pub coins: Vec<Coin>,
    pub coin_idx: usize,
    pub encrypted_bytes: Vec<u8>,
}

impl UserData {
    pub fn new() -> Self {
        let mut addressbook = HashMap::new();
        addressbook.insert(
            String::from("derfarctor (Author)"),
            String::from("donate_3kpznqbuzs3grswcqkzitd5fwky4s5cmyt76wru7kbenfwza7q9c1f1egzhm"),
        );
        UserData {
            password: String::from(""),
            clipboard: Clipboard::new().unwrap(),
            wallets: vec![],
            wallet_idx: 0,
            lookup: HashMap::new(),
            addressbook,
            coins: vec![Coin::nano(), Coin::banano()],
            coin_idx: Coins::NANO,
            encrypted_bytes: vec![],
        }
    }
}
