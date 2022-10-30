use crate::app::coin::Coins;
use crate::app::components::storage::structs::StorageData;
use crate::app::{constants::paths, userdata::UserData};
use crate::crypto::aes::encrypt_bytes;
use cursive::Cursive;
use std::fs;

fn write_storage(encrypted_bytes: Vec<u8>) -> Result<(), String> {
    if let Some(data_dir) = dirs::data_dir() {
        let storage_file = data_dir.join(paths::DATA_DIR).join(paths::STORAGE);
        let write_res = fs::write(&storage_file, encrypted_bytes);
        if write_res.is_err() {
            return Err(format!(
                "Failed to write to {} file at path: {:?}\nError: {:?}",
                paths::STORAGE,
                storage_file,
                write_res.err()
            ));
        }
    }
    Ok(())
}

pub fn save_to_storage(s: &mut Cursive) -> Result<(), String> {
    let data = &s.user_data::<UserData>().unwrap();
    if data.wallets.is_empty() && data.lookup.is_empty() && data.addressbook.is_empty() {
        return write_storage(vec![]);
    }
    let wallets_bytes = bincode::serialize(&data.wallets).unwrap();
    let lookup_bytes = bincode::serialize(&data.lookup).unwrap();
    let addressbook_bytes = bincode::serialize(&data.addressbook).unwrap();
    let mut networks = vec![];
    for coin in &data.coins {
        networks.push(&coin.network)
    }
    let networks_bytes = bincode::serialize(&networks).unwrap();
    let storage_data = StorageData {
        storage_bytes: vec![
            wallets_bytes,
            lookup_bytes,
            addressbook_bytes,
            networks_bytes,
        ],
    };
    let encoded: Vec<u8> = bincode::serialize(&storage_data).unwrap();
    let encrypted_bytes = encrypt_bytes(&encoded, &data.password);
    write_storage(encrypted_bytes)
    //eprintln!("Saved wallets with password: {}", data.password);
}
