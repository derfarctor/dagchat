use super::super::{sendblock::send, sendmessage::send_message};
use super::sent::show_sent;
use crate::app::{
    components::messages::{save::save_messages, structs::SavedMessage},
    constants::{colours::RED, SHOW_TO_DP},
    userdata::UserData,
};
use crate::crypto::conversions::display_to_dp;
use cursive::{
    traits::Resizable,
    views::{Dialog, ProgressBar},
    {utils::markup::StyledString, Cursive},
};
use std::time::SystemTime;

pub fn process_send(s: &mut Cursive, raw: u128, address: String, message: String) {
    let ticks = 1000;
    let cb = s.cb_sink().clone();
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let private_key_bytes = wallet.accounts[wallet.acc_idx].private_key;
    let coin = data.coin.clone();
    s.pop_layer();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                let with_message = !message.is_empty();
                let mut hash = String::from("");
                if !with_message {
                    send(&private_key_bytes, address.clone(), raw, &coin, &counter);
                } else {
                    hash = send_message(
                        &private_key_bytes,
                        address.clone(),
                        raw,
                        message.clone(),
                        &coin,
                        &counter,
                    );
                }
                cb.send(Box::new(move |s| {
                    let mut save_res = Ok(());
                    let data = &mut s.user_data::<UserData>().unwrap();
                    let wallet = &mut data.wallets[data.wallet_idx];
                    let account = &mut wallet.accounts[wallet.acc_idx];
                    account.balance -= raw;
                    if with_message {
                        account.messages.as_mut().unwrap().push(SavedMessage {
                            outgoing: true,
                            address: address.clone(),
                            timestamp: match SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                            {
                                Ok(n) => n.as_secs(),
                                Err(_) => 0u64,
                            },
                            amount: display_to_dp(
                                raw,
                                SHOW_TO_DP,
                                &data.coin.multiplier,
                                &data.coin.ticker,
                            ),
                            hash,
                            plaintext: message.clone(),
                        });
                        save_res = save_messages(s);
                    }
                    show_sent(s, with_message);
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
