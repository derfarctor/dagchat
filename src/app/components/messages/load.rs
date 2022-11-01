use super::{save::create_key, structs::SavedMessage};
use crate::app::constants::paths;
use crate::app::userdata::UserData;
use crate::crypto::aes::decrypt_bytes;
use cursive::Cursive;
use std::fs;

pub fn load_messages(s: &mut Cursive) -> Result<Vec<SavedMessage>, String> {
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let mut messages: Vec<SavedMessage> = vec![];
    let lookup_key = match data.lookup.get(&wallet.accounts[wallet.acc_idx].address) {
        Some(id) => id.to_owned(),
        None => create_key(s)?,
    };
    let data = &mut s.user_data::<UserData>().unwrap();
    let data_dir = dirs::data_dir().unwrap();
    let filename = format!("{}.dagchat", lookup_key);
    let messages_file = data_dir
        .join(paths::DATA_DIR)
        .join(paths::MESSAGES_DIR)
        .join(filename);
    if messages_file.exists() {
        let mut error = String::from("");
        let encrypted_bytes = fs::read(&messages_file).unwrap_or_else(|e| {
            error = format!(
                "Failed to read messages file at path: {:?}\nError: {}",
                messages_file, e
            );
            vec![]
        });
        if !error.is_empty() {
            return Err(error);
        }
        if encrypted_bytes.is_empty() {
            return Ok(vec![]);
        }
        let bytes = decrypt_bytes(&encrypted_bytes, &data.password);

        if let Ok(bytes) = bytes {
            let messages_opt = bincode::deserialize(&bytes[..]);
            if messages_opt.is_ok() {
                messages = messages_opt.unwrap();
            } else {
                let error = format!(
                    "Failed to deserialize messages from file at path: {:?}",
                    messages_file
                );
                return Err(error);
            }
        } else {
            let error = format!(
                "Failed to decrypt messages from file at path: {:?}",
                messages_file
            );
            return Err(error);
        }
    }
    Ok(messages)
}
