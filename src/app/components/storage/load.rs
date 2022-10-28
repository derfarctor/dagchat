use super::structs::*;
use crate::app::components::wallets::ui::primary::show_wallets;
use crate::app::constants::{colours::RED, paths};
use crate::app::userdata::UserData;
use crate::crypto::aes::decrypt_bytes;
use cursive::views::Dialog;
use cursive::{utils::markup::StyledString, Cursive};

pub fn load_with_password(s: &mut Cursive, password: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let bytes = decrypt_bytes(&data.encrypted_bytes, password);
    if bytes.is_err() {
        s.add_layer(Dialog::info("Password was incorrect."));
        return;
    }
    data.password = password.to_string();
    let storage_data: Result<StorageData, _> = bincode::deserialize(&bytes.unwrap()[..]);
    if let Ok(storage_data) = storage_data {
        data.wallets =
            bincode::deserialize(&storage_data.storage_bytes[StorageElements::WALLETS]).unwrap();
        data.lookup =
            bincode::deserialize(&storage_data.storage_bytes[StorageElements::LOOKUP]).unwrap();
        data.addressbook =
            bincode::deserialize(&storage_data.storage_bytes[StorageElements::ADDRESSBOOK])
                .unwrap();
        show_wallets(s);
    } else {
        show_wallets(s);
        s.add_layer(Dialog::info(StyledString::styled(
            format!(
                "Error parsing {} file. File was either corrupted or edited outside of dagchat.",
                paths::STORAGE
            ),
            RED,
        )));
    }
}
