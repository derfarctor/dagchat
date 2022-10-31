use super::super::receiveblock::receive_block;
use crate::app::{
    components::{
        inbox::ui::primary::show_inbox,
        messages::{save::save_messages, structs::SavedMessage},
    },
    constants::{colours::RED, SHOW_TO_DP},
    userdata::UserData,
};
use crate::crypto::conversions::display_to_dp;
use cursive::views::{Dialog, ProgressBar, SelectView, TextView};
use cursive::{traits::Resizable, utils::markup::StyledString, Cursive};
use std::time::SystemTime;

pub fn process_receive(s: &mut Cursive, idx: usize) {
    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let account = &wallet.accounts[wallet.acc_idx];
    let private_key = account.private_key;
    let receivable = &account.receivables[idx];
    let send_block_hash = receivable.hash.clone();

    let amount = receivable.amount;
    let address = account.address.clone();
    let coin = data.coins[data.coin_idx].clone();
    let ticks = 1000;
    let cb = s.cb_sink().clone();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                let mut error = String::from("");
                if let Err(e) = receive_block(
                    &private_key,
                    &send_block_hash,
                    amount,
                    &address,
                    &coin,
                    &counter,
                ) {
                    error = e;
                }
                cb.send(Box::new(move |s| {
                    if !error.is_empty() {
                        s.set_autorefresh(false);
                        s.pop_layer();
                        show_inbox(s);
                        s.add_layer(Dialog::info(StyledString::styled(
                            format!("Receive failed. Error: {}", error),
                            RED,
                        )));
                        return;
                    }
                    let mut select = s.find_name::<SelectView<String>>("select").unwrap();
                    select.remove_item(idx);
                    let mut balance = s.find_name::<TextView>("balance").unwrap();
                    let data = &mut s.user_data::<UserData>().unwrap();
                    let wallet = &mut data.wallets[data.wallet_idx];
                    let account = &mut wallet.accounts[wallet.acc_idx];
                    let receivable = &account.receivables[idx];
                    let send_block_hash = receivable.hash.clone();
                    let amount = receivable.amount;
                    let has_message = { receivable.message.is_some() };
                    let mut save_res = Ok(());
                    if has_message {
                        account.messages.as_mut().unwrap().push(SavedMessage {
                            outgoing: false,
                            address: receivable.source.clone(),
                            timestamp: match SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                            {
                                Ok(n) => n.as_secs(),
                                Err(_) => 0u64,
                            },
                            amount: display_to_dp(
                                amount,
                                SHOW_TO_DP,
                                &coin.multiplier,
                                &coin.ticker,
                            ),
                            hash: send_block_hash,
                            plaintext: receivable.message.as_ref().unwrap().plaintext.clone(),
                        });
                        save_res = save_messages(s);
                    }
                    let data = &mut s.user_data::<UserData>().unwrap();
                    let wallet = &mut data.wallets[data.wallet_idx];
                    let account = &mut wallet.accounts[wallet.acc_idx];
                    account.receivables.remove(idx);
                    account.balance += amount;
                    let bal = display_to_dp(
                        account.balance,
                        SHOW_TO_DP,
                        &data.coins[data.coin_idx].multiplier,
                        &data.coins[data.coin_idx].ticker,
                    );
                    let bal_text = format!("Balance: {}", bal);
                    balance.set_content(StyledString::styled(
                        bal_text,
                        data.coins[data.coin_idx].colour,
                    ));
                    s.pop_layer();
                    s.pop_layer();
                    if save_res.is_err() {
                        s.add_layer(
                            Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                                .title("Failed to save messages"),
                        );
                    }
                }))
                .unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}
