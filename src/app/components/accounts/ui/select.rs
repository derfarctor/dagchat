use crate::app::components::{messages::load::load_messages, receive::load::load_receivables};
use crate::app::userdata::UserData;
use cursive::views::{Dialog, OnEventView, SelectView};
use cursive::Cursive;

pub fn select_account(s: &mut Cursive, _: &str) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("accounts")
        .unwrap();
    let select = eventview.get_inner();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No account selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &mut data.wallets[data.wallet_idx];
    wallet.acc_idx = focus;
    load_current_account(s);
}

pub fn load_current_account(s: &mut Cursive) {
    let messages = load_messages(s);
    let data = &mut s.user_data::<UserData>().unwrap();
    //eprintln!("Loaded messages: {:?}", messages);
    let wallet = &mut data.wallets[data.wallet_idx];
    wallet.accounts[wallet.acc_idx].messages = messages;
    load_receivables(s);
}
