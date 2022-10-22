use crate::app::components::accounts::{structs::Account, ui::primary::show_accounts};
use crate::app::userdata::UserData;
use cursive::views::{Dialog, OnEventView, SelectView};
use cursive::Cursive;

pub fn select_wallet(s: &mut Cursive, _: &str) {
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
    let data = &mut s.user_data::<UserData>().unwrap();
    data.wallet_idx = focus;

    // Generate accounts for saved indexes
    let mut accounts: Vec<Account> = vec![];
    //eprintln!("Saved indexes: {:?}", data.wallets[focus].indexes);
    for index in &data.wallets[focus].indexes {
        accounts.push(Account::with_index(
            &data.wallets[focus],
            *index,
            &data.coin.prefix,
        ));
    }
    data.wallets[focus].accounts = accounts;
    show_accounts(s);
}
