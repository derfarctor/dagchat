use super::process::process_receive;
use crate::app::components::messages::readmessage::read_message;
use crate::app::constants::colours::RED;
use crate::app::{
    clipboard::copy_to_clip,
    constants::{colours::OFF_WHITE, SHOW_TO_DP},
    helpers::go_back,
    userdata::UserData,
};
use crate::crypto::conversions::display_to_dp;
use cursive::views::{Dialog, DummyView, LinearLayout, SelectView, TextView};
use cursive::{
    traits::{Resizable, Scrollable},
    utils::markup::StyledString,
    Cursive,
};

pub fn show_receivable(s: &mut Cursive, _name: &str) {
    let select = s.find_name::<SelectView<String>>("select").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No receivable selected.")),
        Some(focus) => {
            let data = &mut s.user_data::<UserData>().unwrap();
            let wallet = &mut data.wallets[data.wallet_idx];
            let account = &mut wallet.accounts[wallet.acc_idx];
            let receivable = &mut account.receivables[focus];
            let private_key = &account.private_key;
            let coin = &data.coins[data.coin_idx];
            let node_url = &coin.network.node_url;
            let plaintext: String;

            let mut content = LinearLayout::vertical();
            let mut title = format!("{} Receivable", &coin.ticker.trim());
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
                    let read_res = read_message(private_key, target, root_hash, blocks, node_url);
                    if let Ok(plaintext_res) = read_res {
                        plaintext = plaintext_res;
                        message.plaintext = plaintext.clone();
                    } else {
                        s.add_layer(Dialog::info(StyledString::styled(
                            format!("Failed to read message. Error: {}", read_res.err().unwrap()),
                            RED,
                        )));
                        return;
                    }
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
            let colour = coin.colour;
            if !(receivable.amount == 1 && receivable.message.is_some()) {
                receive_label = format!("Receive{}", receive_label);
                let amount = display_to_dp(
                    receivable.amount,
                    SHOW_TO_DP,
                    &coin.multiplier,
                    &coin.ticker,
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
                    .button("Back", go_back)
                    .title(title),
            );
        }
    }
}
