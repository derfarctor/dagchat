use super::structs::WalletsAndLookup;
use crate::app::{constants::paths, userdata::UserData};
use crate::crypto::aes::encrypt_bytes;
use cursive::Cursive;
use std::fs;

fn write_wallets(encrypted_bytes: Vec<u8>) -> Result<(), String> {
    if let Some(data_dir) = dirs::data_dir() {
        let wallets_file = data_dir.join(paths::DATA_DIR).join(paths::WALLETS);
        let write_res = fs::write(&wallets_file, encrypted_bytes);
        if write_res.is_err() {
            return Err(format!(
                "Failed to write to {} file at path: {:?}\nError: {:?}",
                paths::WALLETS,
                wallets_file,
                write_res.err()
            ));
        }
    }
    Ok(())
}

pub fn save_wallets(s: &mut Cursive) -> Result<(), String> {
    let data = &s.user_data::<UserData>().unwrap();
    if data.wallets.is_empty() && data.lookup.is_empty() {
        write_wallets(vec![])?;
        return Ok(());
    }
    let wallets_bytes = bincode::serialize(&data.wallets).unwrap();
    let lookup_bytes = bincode::serialize(&data.lookup).unwrap();
    let wallets_and_lookup = WalletsAndLookup {
        wallets_bytes,
        lookup_bytes,
    };
    let encoded: Vec<u8> = bincode::serialize(&wallets_and_lookup).unwrap();
    let encrypted_bytes = encrypt_bytes(&encoded, &data.password);
    write_wallets(encrypted_bytes)?;
    //eprintln!("Saved wallets with password: {}", data.password);
    Ok(())
}
