use super::structs::*;
use crate::app::components::settings::structs::Network;
use crate::app::components::wallets::ui::primary::show_wallets;
use crate::app::constants::{colours::RED, paths};
use crate::app::constants::{AUTHOR, AUTHOR_ADDR};
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
        let mut errors = String::from("");

        // Load wallets
        if storage_data.storage_bytes.len() > StorageElements::WALLETS {
            if let Ok(wallets) =
                bincode::deserialize(&storage_data.storage_bytes[StorageElements::WALLETS])
            {
                data.wallets = wallets;
            } else {
                errors.push_str(" wallets,");
            }
        } else {
            errors.push_str(" wallets,");
        }

        // Load messages lookup HashMap
        if storage_data.storage_bytes.len() > StorageElements::LOOKUP {
            if let Ok(lookup) =
                bincode::deserialize(&storage_data.storage_bytes[StorageElements::LOOKUP])
            {
                data.lookup = lookup;
            } else {
                errors.push_str(" messages,");
            }
        } else {
            errors.push_str(" messages,");
        }

        // Load address book
        if storage_data.storage_bytes.len() > StorageElements::ADDRESSBOOK {
            if let Ok(addressbook) =
                bincode::deserialize(&storage_data.storage_bytes[StorageElements::ADDRESSBOOK])
            {
                data.addressbook = addressbook;
                data.addressbook
                    .insert(String::from(AUTHOR_ADDR), String::from(AUTHOR));
            } else {
                errors.push_str(" address book,");
            }
        } else {
            errors.push_str(" address book,");
        }

        // Load networks
        if storage_data.storage_bytes.len() > StorageElements::NETWORKS {
            if let Ok(mut networks) = bincode::deserialize::<Vec<Network>>(
                &storage_data.storage_bytes[StorageElements::NETWORKS],
            ) {
                for coin in 0..data.coins.len() {
                    data.coins[coin].network = networks.remove(0);
                }
            } else {
                errors.push_str(" settings,");
            }
        } else {
            errors.push_str(" settings,");
        }

        show_wallets(s);
        if !errors.is_empty() {
            let errors: String = errors.chars().into_iter().take(errors.len() - 1).collect();
            s.add_layer(Dialog::info(StyledString::styled(
                format!(
                    "Error(s) encountered parsing{} from {} - reset to default values.",
                    errors,
                    paths::STORAGE
                ),
                RED,
            )));
        }
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
