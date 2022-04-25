use super::*;

use std::time::SystemTime;

pub fn show_message_info(s: &mut Cursive, _name: &str) {
    let select = s.find_name::<SelectView<String>>("select").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No receivable selected.")),
        Some(focus) => {
            let data = &mut s.user_data::<UserData>().unwrap();
            let wallet = &mut data.wallets[data.wallet_idx];
            let account = &mut wallet.accounts[wallet.acc_idx];
            let receivable = &mut account.receivables[focus];
            let private_key = &account.private_key;
            let node_url = &data.coin.node_url;
            let plaintext: String;

            let mut content = LinearLayout::vertical();
            let mut title = format!("{} Receivable", &data.coin.ticker.trim());
            let mut receive_label = String::from("");
            if receivable.message.is_some() {
                receive_label = String::from(" and mark read");
                title = String::from("Message");
                let mut message = receivable.message.as_mut().unwrap();
                if message.plaintext.is_empty() {
                    // Potential feature: Confirm option with message length in chars (estimated)
                    // removes ability for attacks such as extremely long messages although probably
                    // not an issue. Harder to send a long message than read.
                    let target = &message.head.contents.account;
                    let root_hash = &message.root_hash;
                    let blocks = message.blocks;
                    // Potential feature: Add loading screen + process_message()
                    // time taken to load a (long) message can be noticeable if node
                    // is under load.
                    plaintext = read_message(private_key, target, root_hash, blocks, node_url);
                    message.plaintext = plaintext.clone();
                } else {
                    plaintext = message.plaintext.clone();
                }
                content.add_child(
                    TextView::new(plaintext)
                        .scrollable()
                        .max_width(80)
                        .max_height(6),
                );
                content.add_child(DummyView);
            }
            let colour = data.coin.colour;
            if !(receivable.amount == 1 && receivable.message.is_some()) {
                receive_label = format!("Receive{}", receive_label);
                let amount = display_to_dp(
                    receivable.amount,
                    SHOW_TO_DP,
                    &data.coin.multiplier,
                    &data.coin.ticker,
                );
                content.add_child(TextView::new(StyledString::styled("Amount", colour)));
                content.add_child(TextView::new(StyledString::styled(amount, OFF_WHITE)));
                content.add_child(DummyView);
            } else {
                receive_label = String::from("Mark read");
            }
            let sender = receivable.source.clone();
            content.add_child(TextView::new(StyledString::styled("From", colour)));
            content
                .add_child(TextView::new(StyledString::styled(&sender, OFF_WHITE)).fixed_width(65));

            s.add_layer(
                Dialog::around(content)
                    .button(receive_label, move |s| {
                        process_receive(s, focus);
                    })
                    .button("Copy address", move |s| copy_to_clip(s, sender.clone()))
                    .button("Back", |s| go_back(s))
                    .title(title),
            );
        }
    }
}

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
                let account_info = get_account_info(&target_address, &node_url);
                let mut balance: u128 = 0;
                if account_info.is_some() {
                    balance = get_balance(&account_info.unwrap());
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

fn process_receive(s: &mut Cursive, idx: usize) {
    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let account = &wallet.accounts[wallet.acc_idx];
    let private_key = account.private_key;
    let receivable = &account.receivables[idx];
    let send_block_hash = receivable.hash.clone();

    let amount = receivable.amount;
    let address = account.address.clone();
    let prefix = data.coin.prefix.clone();
    let node_url = data.coin.node_url.clone();
    let ticks = 1000;
    let cb = s.cb_sink().clone();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                counter.tick(100);
                receive_block(
                    &private_key,
                    &send_block_hash,
                    amount,
                    &address,
                    &node_url,
                    &prefix,
                    &counter,
                );
                cb.send(Box::new(move |s| {
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
                                &data.coin.multiplier,
                                &data.coin.ticker,
                            ),
                            hash: send_block_hash.clone(),
                            plaintext: receivable.message.as_ref().unwrap().plaintext.clone(),
                        });
                        save_res = messages::save_messages(s);
                    }
                    let data = &mut s.user_data::<UserData>().unwrap();
                    let wallet = &mut data.wallets[data.wallet_idx];
                    let account = &mut wallet.accounts[wallet.acc_idx];
                    account.receivables.remove(idx);
                    account.balance += amount;
                    let bal = display_to_dp(
                        account.balance,
                        SHOW_TO_DP,
                        &data.coin.multiplier,
                        &data.coin.ticker,
                    );
                    let bal_text = format!("Balance: {}", bal);
                    balance.set_content(StyledString::styled(bal_text, data.coin.colour));
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
