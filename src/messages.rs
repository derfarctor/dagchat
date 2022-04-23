use super::*;
use chrono::prelude::DateTime;
use chrono::{Local, NaiveDateTime, Utc};
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
    //eprintln!("{} : {}", address, hex::encode(random_id));
    data.lookup.insert(address, hex::encode(random_id));
    hex::encode(random_id)
}

pub fn view_messages(s: &mut Cursive) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let messages = &data.acc_messages;
    if messages.is_err() {
        let err_msg = messages.as_ref().err().unwrap().clone();
        s.add_layer(Dialog::info(err_msg));
        return;
    }

    let mut output = StyledString::new();
    for message in messages.as_ref().unwrap().iter().rev() {
        let datetime: DateTime<Local> = DateTime::from(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(message.timestamp as i64, 0),
            Utc,
        ));

        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        let a: &str;
        let b: &str;
        if message.outgoing {
            a = "Sent";
            b = "To:";
        } else {
            a = "Received";
            b = "From:";
        }
        let mut message_info = StyledString::styled(format!("{} at: ", a), OFF_WHITE);
        message_info.append(StyledString::styled(timestamp_str, data.coin.colour));
        message_info.append(StyledString::styled(format!("\n{} ", b), OFF_WHITE));
        message_info.append(StyledString::styled(&message.address, data.coin.colour));
        if !message.plaintext.is_empty() {
            message_info.append(StyledString::styled("\nMessage: ", OFF_WHITE));
            message_info.append(StyledString::styled(&message.plaintext, data.coin.colour));
        }
        message_info.append(StyledString::styled("\nAmount: ", OFF_WHITE));
        message_info.append(StyledString::styled(
            format!("{}\n\n", message.amount),
            data.coin.colour,
        ));
        output.append(message_info);
    }
    s.add_layer(
        Dialog::around(
            TextView::new(output)
                .scrollable()
                .max_width(73)
                .max_height(10),
        )
        .button("Back", |s| go_back(s))
        .title("Message history"),
    );
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
    //eprintln!("Saved messages with password: {}", data.password);
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
            //eprintln!("Changing file password for {}", a);
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
