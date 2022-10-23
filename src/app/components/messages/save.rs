use crate::app::components::wallets::save::save_wallets;
use crate::app::constants::{DATA_DIR_PATH, MESSAGES_DIR_PATH};
use crate::app::userdata::UserData;
use crate::crypto::aes::encrypt_bytes;
use cursive::Cursive;
use rand::RngCore;
use std::fs;

pub fn create_key(s: &mut Cursive) -> Result<String, String> {
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let address = wallet.accounts[wallet.acc_idx].address.clone();
    let mut csprng = rand::thread_rng();
    let mut random_id = [0u8; 32];
    csprng.fill_bytes(&mut random_id);
    //eprintln!("{} : {}", address, hex::encode(random_id));
    data.lookup.insert(address, hex::encode(random_id));
    save_wallets(s)?;
    Ok(hex::encode(random_id))
}

pub fn save_messages(s: &mut Cursive) -> Result<(), String> {
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let data_dir = dirs::data_dir().unwrap();
    let messages_dir = data_dir.join(DATA_DIR_PATH).join(MESSAGES_DIR_PATH);
    let address = &wallet.accounts[wallet.acc_idx].address;
    let lookup_key = match data.lookup.get(address) {
        Some(id) => id.to_owned(),
        None => create_key(s)?,
    };
    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let messages_file = messages_dir.join(format!("{}.dagchat", lookup_key));
    let messages_bytes =
        bincode::serialize(wallet.accounts[wallet.acc_idx].messages.as_ref().unwrap()).unwrap();
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
