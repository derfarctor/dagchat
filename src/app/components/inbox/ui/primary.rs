use super::changerep::show_change_rep;
use super::signmessage::show_sign_message;
use crate::app::{
    clipboard::copy_to_clip,
    constants::{SHOW_TO_DP, VERSION},
    userdata::UserData,
};
use crate::app::{
    components::{
        accounts::ui::primary::show_accounts,
        addressbook::ui::primary::show_addressbook,
        messages::{structs::Filter, ui::primary::show_messages},
        receive::{
            load::load_receivables, ui::primary::show_receivable, ui::process::process_receive,
        },
        send::ui::primary::show_send,
    },
    constants::EMPTY_MSG,
};
use crate::crypto::conversions::display_to_dp;
use cursive::utils::markup::StyledString;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, SelectView, TextView};
use cursive::Cursive;
use cursive::{
    traits::{Nameable, Resizable, Scrollable},
    views::HideableView,
};

pub fn show_inbox(s: &mut Cursive) {
    s.set_autorefresh(false);
    s.pop_layer();
    let data: UserData = s.take_user_data().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let address = wallet.accounts[wallet.acc_idx].address.clone();

    let bal = display_to_dp(
        wallet.accounts[wallet.acc_idx].balance,
        SHOW_TO_DP,
        &data.coins[data.coin_idx].multiplier,
        &data.coins[data.coin_idx].ticker,
    );
    let bal_text = format!("Balance: {}", bal);
    let top_content = LinearLayout::horizontal().child(
        TextView::new(StyledString::styled(
            bal_text,
            data.coins[data.coin_idx].colour,
        ))
        .with_name("balance"),
    );
    let mut select = SelectView::<String>::new().on_submit(show_receivable);

    let mut has_non_msg = false;
    if wallet.accounts[wallet.acc_idx].receivables.is_empty() {
        select.add_item_str(EMPTY_MSG);
    } else {
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
                } else {
                    has_non_msg = true;
                }
            }
            let mut source_parts: Vec<&str> = receivable.source.split('_').collect();
            let source_suffix = String::from('_') + source_parts.pop().unwrap();
            let addr = if data.addressbook.contains_key(&source_suffix) {
                data.addressbook.get(&source_suffix).unwrap()
            } else {
                receivable.source.get(0..11).unwrap()
            };
            tag = format!("{} > {}", addr, tag);
            select.add_item_str(&tag)
        }
    }

    let mut receive_all =
        HideableView::new(Button::new("Receive all", |s| process_receive(s, 0, true)));
    if !has_non_msg {
        receive_all.set_visible(false);
    }
    let buttons = LinearLayout::vertical()
        .child(Button::new("Refresh", load_receivables))
        .child(DummyView)
        .child(Button::new("Send", |s| show_send(s, false)))
        .child(receive_all.with_name("receiveall"))
        .child(DummyView)
        .child(Button::new("Messages log", |s| {
            let filter: Filter = Default::default();
            show_messages(s, filter);
        }))
        .child(Button::new("Address book", show_addressbook))
        .child(Button::new("Copy address", move |s| {
            copy_to_clip(s, address.clone())
        }))
        .child(Button::new("Sign message", show_sign_message))
        .child(Button::new("Change rep", show_change_rep))
        .child(DummyView)
        .child(Button::new("Back", show_accounts));

    s.set_user_data(data);
    s.add_layer(
        HideableView::new(
            Dialog::around(
                LinearLayout::horizontal()
                    .child(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(top_content)
                            .child(DummyView)
                            .child(
                                Dialog::around(
                                    select.with_name("select").scrollable().fixed_height(5),
                                )
                                .padding_lrtb(1, 1, 1, 1)
                                .title("Incoming"),
                            ),
                    )
                    .child(DummyView)
                    .child(DummyView)
                    .child(LinearLayout::vertical().child(DummyView).child(buttons)),
            )
            .title(format!("dagchat {}", VERSION)),
        )
        .with_name("hideable"),
    );
}
