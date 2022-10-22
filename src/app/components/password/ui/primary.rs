use super::super::changemessage::change_message_passwords;
use crate::app::components::wallets::{save::save_wallets, ui::primary::show_wallets};
use crate::app::{constants::colours::RED, userdata::UserData};
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Dialog, DummyView, EditView, LinearLayout, TextView};
use cursive::{align::HAlign, utils::markup::StyledString, Cursive};

pub fn set_password<F: 'static>(s: &mut Cursive, on_success: F)
where
    F: Fn(&mut Cursive),
{
    let warning = StyledString::styled(
        "Always backup or write down your mnemonics, seeds or keys elsewhere in case you forget your password.", RED);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(TextView::new("Enter password"))
                .child(EditView::new().secret().with_name("password"))
                .child(DummyView)
                .child(TextView::new("Confirm password"))
                .child(EditView::new().secret().with_name("confirm"))
                .child(DummyView)
                .child(TextView::new(warning)),
        )
        .h_align(HAlign::Center)
        .button("Submit", move |s| {
            let password = s
                .call_on_name("password", |view: &mut EditView| view.get_content())
                .unwrap();
            let confirmed = s
                .call_on_name("confirm", |view: &mut EditView| view.get_content())
                .unwrap();
            if password.is_empty() {
                s.add_layer(Dialog::info("Password can't be blank."));
                return;
            }
            if password != confirmed {
                s.add_layer(Dialog::info("Passwords did not match."));
                return;
            }
            let msg_save_res = change_message_passwords(s, &password);
            let data = &mut s.user_data::<UserData>().unwrap();
            data.password = password.to_string();
            let acc_save_res = save_wallets(s);
            s.pop_layer();
            if acc_save_res.is_ok() && msg_save_res.is_ok() {
                on_success(s);
            } else if acc_save_res.is_err() {
                show_wallets(s);
                s.add_layer(Dialog::info(StyledString::styled(acc_save_res.err().unwrap(),
                    RED,
                )).title("Fatal error saving wallets"));
            } else if msg_save_res.is_err() {
                show_wallets(s);
                s.add_layer(Dialog::info(StyledString::styled(msg_save_res.err().unwrap(),
                    RED,
                )).title("Fatal error saving messages"));
            }
        })
        .button("Info", |s| {
            let content = "\nThe password you are setting up for dagchat is used to encrypt your wallets, messages (If 'Encrypt and save' messages setting is selected) and address book when they are saved on your device. It should be strong and contain a range of characters (UPPERCASE, lowercase, numb3rs and symbo!s). Without this password, dagchat will not be able to decrypt any of your saved wallets, messages or address book.";
            s.add_layer(Dialog::info(content).title("What is this password?").max_width(80));
        })
        .title("Create a password for dagchat")
        .max_width(80),
    );
}
