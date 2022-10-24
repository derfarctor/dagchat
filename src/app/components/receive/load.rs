use crate::app::components::inbox::ui::primary::show_inbox;
use crate::app::userdata::UserData;
use crate::rpc::{accountinfo::*, incoming::find_incoming};
use cursive::traits::Resizable;
use cursive::views::{Dialog, ProgressBar};
use cursive::Cursive;

pub fn load_receivables(s: &mut Cursive) {
    let ticks = 1000;

    let cb = s.cb_sink().clone();

    let data = &s.user_data::<UserData>().unwrap();
    let node_url = data.coin.node_url.clone();
    let wallet = &data.wallets[data.wallet_idx];
    let target_address = wallet.accounts[wallet.acc_idx].address.clone();
    s.pop_layer();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                let mut balance: u128 = 0;
                if let Ok(account_info) = get_account_info(&target_address, &node_url) {
                    balance = get_balance(&account_info);
                }
                counter.tick(100);
                let receivables = find_incoming(&target_address, &node_url, &counter);
                cb.send(Box::new(move |s| {
                    let data = &mut s.user_data::<UserData>().unwrap();
                    let wallet = &mut data.wallets[data.wallet_idx];
                    let account = &mut wallet.accounts[wallet.acc_idx];
                    account.receivables = receivables;
                    account.balance = balance;
                    show_inbox(s);
                }))
                .unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}
