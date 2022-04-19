use super::*;
use rand::RngCore;

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedMessage {
    // If false, was incoming
    pub outgoing: bool,
    pub address: String,
    pub timestamp: u64,
    pub amount: String,
    pub plaintext: String,
    pub hash: String,
}

pub fn create_key(s: &mut Cursive) -> String {
    let data = &mut s.user_data::<UserData>().unwrap();
    let address = data.accounts[data.acc_idx].address.clone();
    let mut csprng = rand::thread_rng();
    let mut random_id = [0u8; 32];
    csprng.fill_bytes(&mut random_id);
    eprintln!("{} : {}", address, hex::encode(random_id));
    data.lookup.insert(address, hex::encode(random_id));
    hex::encode(random_id)
}

pub fn load_messages(s: &mut Cursive) -> Result<Vec<SavedMessage>, String> {
    let data = &mut s.user_data::<UserData>().unwrap();
    let mut messages: Vec<SavedMessage> = vec![];
    let lookup_key = match data.lookup.get(&data.accounts[data.acc_idx].address) {
        Some(id) => id.to_owned(),
        None => create_key(s),
    };
    let data = &mut s.user_data::<UserData>().unwrap();
    let data_dir = dirs::data_dir().unwrap();
    let filename = format!("{}.dagchat", lookup_key);
    let messages_file = data_dir
        .join(DATA_DIR_PATH)
        .join(MESSAGES_DIR_PATH)
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
        let bytes = dcutil::decrypt_bytes(&encrypted_bytes, &data.password);
        let messages_opt = bincode::deserialize(&bytes.unwrap()[..]);
        if messages_opt.is_err() {
            let error = format!(
                "Failed to decode messages from file at path: {:?}",
                messages_file
            );
            return Err(error);
        }
        messages = messages_opt.unwrap();
    }

    Ok(messages)
}

pub fn save_messages(s: &mut Cursive) -> Result<(), String> {
    let data = &mut s.user_data::<UserData>().unwrap();
    let data_dir = dirs::data_dir().unwrap();
    let messages_dir = data_dir.join(DATA_DIR_PATH).join(MESSAGES_DIR_PATH);
    let address = &data.accounts[data.acc_idx].address;
    let lookup_key = match data.lookup.get(address) {
        Some(id) => id.to_owned(),
        None => create_key(s),
    };
    let data = &mut s.user_data::<UserData>().unwrap();
    let messages_file = messages_dir.join(format!("{}.dagchat", lookup_key));
    let messages_bytes = bincode::serialize(data.acc_messages.as_ref().unwrap()).unwrap();
    let encrypted_bytes = encrypt_bytes(&messages_bytes, &data.password);
    let write_res = fs::write(&messages_file, encrypted_bytes);
    if write_res.is_err() {
        return Err(format!(
            "Failed to write to messages file at path: {:?}\nError: {:?}",
            messages_file,
            write_res.err()
        ));
    }
    eprintln!("Saved messages with password: {}", data.password);
    Ok(())
}

pub fn change_message_passwords(s: &mut Cursive, new_password: &str) -> Result<(), String> {
    let data = &mut s.user_data::<UserData>().unwrap();
    let data_dir = dirs::data_dir().unwrap();
    let messages_dir = data_dir.join(DATA_DIR_PATH).join(MESSAGES_DIR_PATH);

    for (a, lookup_key) in data.lookup.iter() {
        let filename = format!("{}.dagchat", lookup_key);
        let messages_file = messages_dir.join(filename);
        if messages_file.exists() {
            eprintln!("Changing file password for {}", a);
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
    eprintln!(
        "Saved and changed all new messages with password: {}",
        new_password
    );
    Ok(())
}
