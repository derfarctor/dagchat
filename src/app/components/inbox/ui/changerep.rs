use super::primary::show_inbox;
use crate::app::components::addressbook::ui::primary::show_addressbook;
use crate::app::{clipboard::paste_clip, themes::get_subtitle_colour, userdata::UserData};
use crate::crypto::address::validate_address;
use crate::rpc::{accountinfo::get_account_info, changerep::change_rep};
use cursive::traits::{Nameable, Resizable};
use cursive::utils::markup::StyledString;
use cursive::views::{Button, Dialog, DummyView, HideableView, LinearLayout, TextArea, TextView};
use cursive::Cursive;

pub fn show_change_rep(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let account = &wallet.accounts[wallet.acc_idx];
    let private_key = account.private_key;
    let coin = data.coin.clone();
    let address = account.address.clone();
    let sub_title_colour = get_subtitle_colour(coin.colour);
    s.add_layer(
        HideableView::new(
        Dialog::around(
            LinearLayout::vertical()
            .child(DummyView)
                .child(TextView::new(StyledString::styled(
                    "Representative address",
                    sub_title_colour,
                )))
                .child(TextArea::new().with_name("address").max_width(66))
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Paste", |s| {
                            s.call_on_name("address", |view: &mut TextArea| {
                                view.set_content(paste_clip());
                            })
                            .unwrap();
                        }))
                        .child(Button::new("Address book", 
                            show_addressbook
                        )),
                )
                .child(DummyView)
                .child(LinearLayout::horizontal()
                .child(Button::new("Change", move |s| {
                    let mut rep_address = String::from("");
                    s.call_on_name("address", |view: &mut TextArea| {
                        rep_address = String::from(view.get_content());
                    });
                    if rep_address.is_empty() {
                        s.add_layer(Dialog::info(
                            "You must provide an address to change representative to!"
                        ));
                        return;
                    }
                    let valid = validate_address(&rep_address);
                    if !valid {
                        s.add_layer(Dialog::info("The representative's address is invalid."));
                        return;
                    }
                    let account_info_opt = get_account_info(&address, &coin.node_url);
                    if account_info_opt.is_ok() {
                        s.add_layer(Dialog::info(format!("You can't change representatives until you open your account by receiving some {}.", coin.name)));
                        return;
                    }
                    let account_info = account_info_opt.unwrap();
                    change_rep(&private_key, account_info, &rep_address, &coin);
                    s.pop_layer();
                    show_inbox(s);
                    s.add_layer(Dialog::info("Successfully changed representative!"));
                }))
                .child(Button::new("Back", show_inbox))),
        )
        .title("Change representative")).with_name("hideable"),
    );
}
