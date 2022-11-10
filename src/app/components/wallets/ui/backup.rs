use crate::app::{clipboard::*, constants::colours::RED, helpers::go_back, userdata::UserData};
use cursive::views::{Dialog, DummyView, LinearLayout, OnEventView, SelectView, TextView};
use cursive::{align::HAlign, traits::*, utils::markup::StyledString, Cursive};

pub fn backup_wallet(s: &mut Cursive) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("wallets")
        .unwrap();
    let select = eventview.get_inner();
    let selected_idx = match select.selected_id() {
        None => {
            s.add_layer(Dialog::info("No wallet selected."));
            return;
        }
        Some(focus) => focus,
    };
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[selected_idx];
    let mut content = Dialog::around(LinearLayout::vertical().child(DummyView).child(
        TextView::new(StyledString::styled(
            "Make sure you are in a safe location before viewing your mnemonic, seed or key.",
            RED,
        )),
    ))
    .h_align(HAlign::Center)
    .title("Backup wallet");
    if !wallet.mnemonic.is_empty() {
        let mnemonic = wallet.mnemonic.clone();
        content.add_button("Mnemonic", move |s| {
            let mnemonic = mnemonic.clone();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(DummyView)
                        .child(TextView::new(&mnemonic)),
                )
                .h_align(HAlign::Center)
                .button("Copy", move |s| {
                    s.pop_layer();
                    s.pop_layer();
                    copy_to_clip(s, mnemonic.clone())
                })
                .button("Back", go_back)
                .title("Mnemonic")
                .max_width(80),
            );
        });
        let seed = hex::encode(wallet.seed);
        content.add_button("Hex seed", move |s| {
            let seed = seed.clone();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(DummyView)
                        .child(TextView::new(&seed)),
                )
                .h_align(HAlign::Center)
                .button("Copy", move |s| {
                    s.pop_layer();
                    s.pop_layer();
                    copy_to_clip(s, seed.clone())
                })
                .button("Back", go_back)
                .title("Seed"),
            );
        });
    } else {
        let private_key = hex::encode(wallet.seed);
        content.add_button("Private key", move |s| {
            let private_key = private_key.clone();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(DummyView)
                        .child(TextView::new(&private_key)),
                )
                .h_align(HAlign::Center)
                .button("Copy", move |s| {
                    s.pop_layer();
                    s.pop_layer();
                    copy_to_clip(s, private_key.clone())
                })
                .button("Back", go_back)
                .title("Private key"),
            );
        });
    }
    content.add_button("Back", go_back);

    s.add_layer(content.max_width(80));
}
