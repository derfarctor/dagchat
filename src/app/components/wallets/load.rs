use super::{structs::WalletsAndLookup, ui::primary::show_wallets};
use crate::app::{
    constants::{colours::RED, WALLETS_PATH},
    userdata::UserData,
};
use crate::crypto::aes::decrypt_bytes;
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout};
use cursive::{traits::*, utils::markup::StyledString, Cursive};
use std::{fs, path::PathBuf};

pub fn load_wallets(s: &mut Cursive, data_path: PathBuf) {
    s.pop_layer();
    let wallets_file = data_path.join(WALLETS_PATH);
    if wallets_file.exists() {
        let encrypted_bytes = fs::read(&wallets_file).unwrap_or_else(|e| {
            let content = format!(
                "Failed to read {} file at path: {:?}\nError: {}",
                WALLETS_PATH, wallets_file, e
            );
            s.add_layer(Dialog::info(content));
            vec![]
        });
        if encrypted_bytes.is_empty() {
            show_wallets(s);
            return;
        }
        let data = &mut s.user_data::<UserData>().unwrap();
        data.encrypted_bytes = encrypted_bytes;
        s.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .child(DummyView)
                    .child(
                        EditView::new()
                            .secret()
                            .on_submit(move |s, password| {
                                load_with_password(s, password);
                            })
                            .with_name("password"),
                    )
                    .child(DummyView)
                    .child(Button::new("Submit", move |s| {
                        let password = s
                            .call_on_name("password", |view: &mut EditView| view.get_content())
                            .unwrap();
                        load_with_password(s, &password);
                    })),
            )
            .title("Enter dagchat password")
            .max_width(80),
        );
    } else {
        show_wallets(s);
    }
}

fn load_with_password(s: &mut Cursive, password: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let bytes = decrypt_bytes(&data.encrypted_bytes, password);
    if bytes.is_err() {
        s.add_layer(Dialog::info("Password was incorrect."));
        return;
    }
    data.password = password.to_string();

    let wallets_and_lookup_res: Result<WalletsAndLookup, _> =
        bincode::deserialize(&bytes.unwrap()[..]);
    if let Ok(wallets_and_lookup) = wallets_and_lookup_res {
        data.wallets = bincode::deserialize(&wallets_and_lookup.wallets_bytes).unwrap();
        data.lookup = bincode::deserialize(&wallets_and_lookup.lookup_bytes).unwrap();
        show_wallets(s);
    } else {
        show_wallets(s);
        s.add_layer(Dialog::info(StyledString::styled(
            format!(
                "Error parsing {} file. File was either corrupted or edited outside of dagchat.",
                WALLETS_PATH
            ),
            RED,
        )));
    }
}
