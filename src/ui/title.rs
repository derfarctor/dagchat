//
use super::super::wallets;
use super::themes::*;
use crate::util::constants::*;
//
use crate::util::coin::Coin;
use crate::util::userdata::UserData;
use cursive::align::HAlign;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, RadioGroup};
use cursive::Cursive;
use std::fs;

pub fn show_title(s: &mut Cursive) {
    let data = UserData::new();
    s.set_user_data(data);

    set_theme(s, "nano", false);
    let mut theme_group: RadioGroup<bool> = RadioGroup::new();

    let mut coin_group: RadioGroup<String> = RadioGroup::new();

    let radios = LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
                .child(coin_group.button(String::from("nano"), "nano").selected())
                .child(coin_group.button(String::from("banano"), "banano")),
        )
        .child(DummyView)
        .child(
            LinearLayout::vertical()
                .child(theme_group.button(false, "Modest").selected())
                .child(theme_group.button(true, "Vibrant")),
        );

    let button = Button::new_raw("Start", move |s| {
        let coin = coin_group.selection();
        let vibrant = theme_group.selection();
        set_theme(s, &*coin, *vibrant);
        if *coin == "banano" {
            s.with_user_data(|data: &mut UserData| {
                data.coin = Coin::banano();
            });
        }
        check_setup(s);
    });

    s.add_layer(
        Dialog::new()
            .content(
                LinearLayout::vertical()
                    .child(DummyView)
                    .child(button)
                    .child(DummyView)
                    .child(radios),
            )
            .title(format!("dagchat {}", VERSION))
            .h_align(HAlign::Center),
    );
}

fn check_setup(s: &mut Cursive) {
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
        wallets::load_wallets(s, dagchat_dir);
    } else {
        s.add_layer(Dialog::info(
            "Error locating the application data folder on your system.",
        ));
    }
}
