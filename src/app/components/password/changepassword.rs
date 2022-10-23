use crate::app::constants::paths;
use crate::app::userdata::UserData;
use crate::crypto::aes::{decrypt_bytes, encrypt_bytes};
use cursive::Cursive;
use std::fs;

pub fn change_password(s: &mut Cursive, new_password: &str) -> Result<(), String> {
    let data = &mut s.user_data::<UserData>().unwrap();
    let data_dir = dirs::data_dir().unwrap();
    let messages_dir = data_dir.join(paths::DATA_DIR).join(paths::MESSAGES_DIR);

    for (_a, lookup_key) in data.lookup.iter() {
        let filename = format!("{}.dagchat", lookup_key);
        let messages_file = messages_dir.join(filename);
        if messages_file.exists() {
            //eprintln!("Changing file password for {}", _a);
            let mut error = String::from("");
            let encrypted_bytes = fs::read(&messages_file).unwrap_or_else(|e| {
                error = format!(
                    "Failed to read messages file at path: {:?}\nError: {}",
                    messages_file, e
                );
                vec![]
            });
            if encrypted_bytes.is_empty() {
                continue;
            }
            let decrypted_bytes = decrypt_bytes(&encrypted_bytes, &data.password)?;
            let reencrypted_bytes = encrypt_bytes(&decrypted_bytes, new_password);
            let write_res = fs::write(&messages_file, reencrypted_bytes);
            if write_res.is_err() {
                return Err(format!(
                    "Failed to write to messages file at path: {:?}\nError: {:?}",
                    messages_file,
                    write_res.err()
                ));
            }
        }
    }
    //eprintln!(
    //    "Saved and changed all new messages with password: {}",
    //    new_password
    //);
    Ok(())
}
