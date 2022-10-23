use super::super::{save::save_wallets, ui::primary::show_wallets};
use super::backup::backup_wallet;
use crate::app::{
    constants::{colours::RED, paths},
    userdata::UserData,
};
use cursive::views::{Dialog, DummyView, LinearLayout, OnEventView, SelectView, TextView};
use cursive::{align::HAlign, utils::markup::StyledString, Cursive};

pub fn remove_wallet(s: &mut Cursive) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("wallets")
        .unwrap();
    let select = eventview.get_inner();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No wallet selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let warning = StyledString::styled(
        "If you have not backed up this wallet, all of its accounts will be lost forever.",
        RED,
    );
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(TextView::new(warning))
                .child(DummyView),
        )
        .h_align(HAlign::Center)
        .button("Back", |s| {
            s.pop_layer();
        })
        .button("Backup", backup_wallet)
        .button("Confirm", move |s| {
            let data = &mut s.user_data::<UserData>().unwrap();
            let wallet = &data.wallets[focus];

            // Remove account addresses from lookup if they
            // have no messages linked.
            for account in &wallet.accounts {
                let data_dir = dirs::data_dir().unwrap();
                let messages_dir = data_dir.join(paths::DATA_DIR).join(paths::MESSAGES_DIR);
                if data.lookup.contains_key(&account.address) {
                    let filename =
                        format!("{}.dagchat", data.lookup.get(&account.address).unwrap());
                    let messages_file = messages_dir.join(filename);
                    if !messages_file.exists() {
                        data.lookup.remove(&account.address);
                    }
                }
            }
            data.wallets.remove(focus);
            let save_res = save_wallets(s);
            s.pop_layer();
            s.pop_layer();
            show_wallets(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving wallets data"),
                );
            }
        })
        .title("Confirm wallet deletion"),
    );
}
