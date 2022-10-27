use crate::app::{constants::paths, userdata::UserData};
use crate::crypto::aes::encrypt_bytes;
use cursive::Cursive;
use std::fs;

fn write_addressbook(encrypted_bytes: Vec<u8>) -> Result<(), String> {
    if let Some(data_dir) = dirs::data_dir() {
        let addressbook_file = data_dir.join(paths::DATA_DIR).join(paths::ADDRESS_BOOK);
        let write_res = fs::write(&addressbook_file, encrypted_bytes);
        if write_res.is_err() {
            return Err(format!(
                "Failed to write to {} file at path: {:?}\nError: {:?}",
                paths::ADDRESS_BOOK,
                wallets_file,
                write_res.err()
            ));
        }
    }
    Ok(())
}

pub fn save_addressbook(s: &mut Cursive) -> Result<(), String> {
    let data = &s.user_data::<UserData>().unwrap();
    if data.addressbook.is_empty() {
        write_addressbook(vec![])?;
        return Ok(());
    }
    let addressbook_bytes = bincode::serialize(&data.wallets).unwrap();
    let encrypted_bytes = encrypt_bytes(&addressbook_bytes, &data.password);
    write_addressbook(encrypted_bytes)
    //eprintln!("Saved wallets with password: {}", data.password);
}
