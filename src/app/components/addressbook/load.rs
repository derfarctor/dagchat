use super::{structs::WalletsAndLookup, ui::primary::show_wallets};
use crate::app::{
    constants::{colours::RED, paths},
    userdata::UserData,
};
use crate::crypto::aes::decrypt_bytes;
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout};
use cursive::{traits::*, utils::markup::StyledString, Cursive};
use std::{fs, path::PathBuf};

pub fn load_with_password(s: &mut Cursive, password: &str) {   
    let data = &mut s.user_data::<UserData>().unwrap();
    let bytes = decrypt_bytes(&data.encrypted_bytes, password);
    if bytes.is_err() {
        s.add_layer(Dialog::info(StyledString::styled(
            format!(
                "Error reading address book at path {} The file was corrupted.",
                paths::ADDRESSBOOK
            ),
            RED,
        )));
        return;
    }
    data.password = password.to_string();

    let addressbook: Result<HashMap<String, String>, _> =
        bincode::deserialize(&bytes.unwrap()[..]);

    if let Ok(addressbook) = addressbook {
        data.addressbook = bincode::deserialize(&addressbook).unwrap();
        show_addressbook(s);
    } else {
        s.add_layer(Dialog::info(StyledString::styled(
            format!(
                "Error parsing {} file. File was either corrupted or edited outside of dagchat.",
                paths::ADDRESSBOOK
            ),
            RED,
        )));
    }
}
