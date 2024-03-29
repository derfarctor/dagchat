use super::super::add::add_account;
use super::primary::show_accounts;
use crate::app::components::storage::save::save_to_storage;
use crate::app::constants::colours::RED;
use crate::app::userdata::UserData;
use cursive::utils::markup::StyledString;
use cursive::views::{Dialog, DummyView, EditView, LinearLayout, TextView};
use cursive::{align::HAlign, traits::Nameable, Cursive};

pub fn add_index(s: &mut Cursive) {
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(TextView::new("Account index (0 - 4,294,967,295)"))
                .child(EditView::new().on_submit(process_idx).with_name("index")),
        )
        .h_align(HAlign::Center)
        .button("Submit", move |s| {
            let idx = s
                .call_on_name("index", |view: &mut EditView| view.get_content())
                .unwrap();
            process_idx(s, &idx);
        })
        .button("Back", |s| {
            s.pop_layer();
        })
        .title("Show account with index"),
    );
}

fn process_idx(s: &mut Cursive, idx: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let prefix = &data.coins[data.coin_idx].prefix.clone();
    let wallet = &data.wallets[data.wallet_idx];
    let index_res: Result<u32, _> = idx.parse();

    if let Ok(index) = index_res {
        if wallet.indexes.contains(&index) {
            s.add_layer(Dialog::info("This account has already been added!"));
        } else {
            add_account(s, Some(index_res.unwrap()), prefix);
            let save_res = save_to_storage(s);
            s.pop_layer();
            s.pop_layer();
            show_accounts(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving wallets data."),
                );
            }
        }
    } else {
        s.add_layer(Dialog::info(
            "Error: index was not an integer within the valid range.",
        ));
    }
}
