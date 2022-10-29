use crate::app::components::storage::ui::primary::show_get_password;
use crate::app::constants::paths;
use cursive::views::Dialog;
use cursive::Cursive;
use std::fs;

pub fn check_setup(s: &mut Cursive) {
    if let Some(data_dir) = dirs::data_dir() {
        let dagchat_dir = data_dir.join(paths::DATA_DIR);
        let messages_dir = dagchat_dir.join(paths::MESSAGES_DIR);
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
        show_get_password(s, dagchat_dir);
    } else {
        s.add_layer(Dialog::info(
            "Error locating the application data folder on your system.",
        ));
    }
}
