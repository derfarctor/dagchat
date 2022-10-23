use crate::app::userdata::UserData;
use cursive::views::{Dialog, OnEventView, SelectView};
use cursive::Cursive;

pub fn remove_account(s: &mut Cursive) {
    let mut eventview = s
        .find_name::<OnEventView<SelectView<String>>>("accounts")
        .unwrap();
    let select = eventview.get_inner_mut();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No account selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &mut data.wallets[data.wallet_idx];
    wallet.accounts.remove(focus);
    wallet.indexes.remove(focus);
    select.remove_item(focus);
}
