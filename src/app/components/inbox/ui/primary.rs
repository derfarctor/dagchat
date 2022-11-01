use super::changerep::show_change_rep;
use crate::app::components::{
    accounts::ui::primary::show_accounts,
    messages::{structs::Filter, ui::primary::show_messages},
    receive::{load::load_receivables, ui::primary::show_receivable},
    send::ui::primary::show_send,
};
use crate::app::{
    clipboard::copy_to_clip,
    constants::{SHOW_TO_DP, VERSION},
    userdata::UserData,
};
use crate::crypto::conversions::display_to_dp;
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, SelectView, TextView};
use cursive::Cursive;

pub fn show_inbox(s: &mut Cursive) {
    s.set_autorefresh(false);
    s.pop_layer();
    let data: UserData = s.take_user_data().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let address = wallet.accounts[wallet.acc_idx].address.clone();
    let send_label = format!("Send {}", data.coins[data.coin_idx].name);
    let buttons = LinearLayout::vertical()
        .child(Button::new("Refresh", load_receivables))
        .child(DummyView)
        .child(Button::new(send_label, |s| show_send(s, false)))
        .child(DummyView)
        .child(Button::new("Send message", |s| show_send(s, true)))
        .child(Button::new("Messages log", |s| {
            let filter: Filter = Default::default();
            show_messages(s, filter);
        }))
        .child(DummyView)
        .child(Button::new("Copy address", move |s| {
            copy_to_clip(s, address.clone())
        }))
        .child(Button::new("Change rep", show_change_rep))
        .child(DummyView)
        .child(Button::new("Back", show_accounts));

    let select = SelectView::<String>::new()
        .on_submit(show_receivable)
        .with_name("select")
        .scrollable()
        .fixed_height(5);

    let bal = display_to_dp(
        wallet.accounts[wallet.acc_idx].balance,
        SHOW_TO_DP,
        &data.coins[data.coin_idx].multiplier,
        &data.coins[data.coin_idx].ticker,
    );
    let bal_text = format!("Balance: {}", bal);
    let bal_content = TextView::new(StyledString::styled(
        bal_text,
        data.coins[data.coin_idx].colour,
    ))
    .with_name("balance");
    s.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(
                    LinearLayout::vertical()
                        .child(DummyView)
                        .child(bal_content)
                        .child(DummyView)
                        .child(
                            Dialog::around(select)
                                .padding_lrtb(1, 1, 1, 1)
                                .title("Incoming"),
                        ),
                )
                .child(DummyView)
                .child(DummyView)
                .child(LinearLayout::vertical().child(DummyView).child(buttons)),
        )
        .title(format!("dagchat {}", VERSION)),
    );

    for receivable in &wallet.accounts[wallet.acc_idx].receivables {
        let mut tag;
        if receivable.amount == 1 && receivable.message.is_some() {
            tag = String::from("Message");
        } else {
            tag = display_to_dp(
                receivable.amount,
                SHOW_TO_DP,
                &data.coins[data.coin_idx].multiplier,
                &data.coins[data.coin_idx].ticker,
            );
            if receivable.message.is_some() {
                tag = format!("{} + Msg", tag);
            }
        }
        let addr = if data.addressbook.contains_key(&receivable.source) {
            data.addressbook.get(&receivable.source).unwrap()
        } else {
            receivable.source.get(0..11).unwrap()
        };
        tag = format!("{} > {}", addr, tag);
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(&tag)
        });
    }
    s.set_user_data(data);
}
