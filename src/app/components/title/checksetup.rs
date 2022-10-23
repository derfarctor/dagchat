use crate::app::components::wallets::load::load_wallets;
use crate::app::constants::{DATA_DIR_PATH, MESSAGES_DIR_PATH};
use cursive::views::Dialog;
use cursive::Cursive;
use std::fs;

pub fn check_setup(s: &mut Cursive) {
    if let Some(data_dir) = dirs::data_dir() {
        let dagchat_dir = data_dir.join(DATA_DIR_PATH);
        let messages_dir = dagchat_dir.join(MESSAGES_DIR_PATH);
        if !dagchat_dir.exists() {
            fs::create_dir(&dagchat_dir).unwrap_or_else(|e| {
                let content = format!(
                    "Failed to create a data folder for dagchat at path: {:?}\nError: {}",
                    dagchat_dir, e
                );
                s.add_layer(Dialog::info(content))
            });
            if !dagchat_dir.exists() {
                return;
            }
        }
        if !messages_dir.exists() {
            fs::create_dir(&messages_dir).unwrap_or_else(|e| {
                let content = format!(
                    "Failed to create a messages folder for dagchat at path: {:?}\nError: {}",
                    messages_dir, e
                );
                s.add_layer(Dialog::info(content))
            });
            if !messages_dir.exists() {
                return;
            }
        }
        load_wallets(s, dagchat_dir);
    } else {
        s.add_layer(Dialog::info(
            "Error locating the application data folder on your system.",
        ));
    }
}
