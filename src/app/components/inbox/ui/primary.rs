use super::changerep::show_change_rep;
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
        receive::{load::load_receivables, ui::primary::show_receivable},
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
    let buttons = LinearLayout::vertical()
        .child(Button::new("Refresh", load_receivables))
        .child(DummyView)
        .child(Button::new("Send", |s| show_send(s, false)))
        .child(DummyView)
        .child(Button::new("Messages log", |s| {
            let filter: Filter = Default::default();
            show_messages(s, filter);
        }))
        .child(Button::new("Address book", show_addressbook))
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
        HideableView::new(
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
        )
        .with_name("hideable"),
    );

    if wallet.accounts[wallet.acc_idx].receivables.is_empty() {
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(EMPTY_MSG);
        });
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
            s.call_on_name("select", |view: &mut SelectView<String>| {
                view.add_item_str(&tag)
            });
        }
    }

    s.set_user_data(data);
}
