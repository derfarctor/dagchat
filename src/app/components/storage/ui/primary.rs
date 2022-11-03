use super::super::load::load_with_password;
use crate::app::components::title::ui::primary::show_title;
use crate::app::components::wallets::ui::primary::show_wallets;
use crate::app::constants::paths;
use crate::app::userdata::UserData;
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout};
use cursive::Cursive;

use std::fs;
use std::path::PathBuf;

pub fn show_get_password(s: &mut Cursive, data_path: PathBuf) {
    s.pop_layer();
    let storage_file = data_path.join(paths::STORAGE);
    if storage_file.exists() {
        let encrypted_bytes = fs::read(&storage_file).unwrap_or_else(|e| {
            let content = format!(
                "Failed to read {} file at path: {:?}\nError: {}",
                paths::STORAGE,
                storage_file,
                e
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
                    .child(
                        LinearLayout::horizontal()
                            .child(Button::new("Submit", move |s| {
                                let password = s
                                    .call_on_name("password", |view: &mut EditView| {
                                        view.get_content()
                                    })
                                    .unwrap();
                                load_with_password(s, &password);
                            }))
                            .child(DummyView)
                            .child(Button::new("Back", |s| {
                                s.pop_layer();
                                show_title(s);
                            })),
                    ),
            )
            .title("Enter password")
            .max_width(80),
        );
    } else {
        show_wallets(s);
    }
}
