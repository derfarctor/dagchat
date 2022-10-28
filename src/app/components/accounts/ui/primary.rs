use super::super::add::add_account;
use super::{add::add_index, remove::remove_account, select::select_account};
use crate::app::components::{storage::save::save_to_storage, wallets::ui::primary::show_wallets};
use crate::app::constants::colours::RED;
use crate::app::userdata::UserData;
use cursive::event::{Event, EventResult, EventTrigger, MouseEvent};
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, OnEventView, SelectView};
use cursive::Cursive;

pub fn show_accounts(s: &mut Cursive) {
    s.pop_layer();
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let wallet_name = wallet.name.clone();
    let prefix = data.coin.prefix.clone();

    let mut buttons = LinearLayout::horizontal().child(DummyView);
    if !wallet.mnemonic.is_empty() {
        buttons.add_child(Button::new("Show next", move |s| {
            add_account(s, None, &prefix);
            let save_res = save_to_storage(s);
            s.pop_layer();
            show_accounts(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving wallets data."),
                );
            }
        }));
        buttons.add_child(DummyView);
        buttons.add_child(Button::new("Show index", add_index));
        buttons.add_child(DummyView);
        buttons.add_child(Button::new("Hide", |s| {
            let data = &mut s.user_data::<UserData>().unwrap();
            let wallet = &data.wallets[data.wallet_idx];
            if wallet.accounts.len() == 1 {
                s.add_layer(Dialog::info("You can't hide your final account!"));
                return;
            }
            remove_account(s);
            let save_res = save_to_storage(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving wallets data."),
                );
            }
        }));
        buttons.add_child(DummyView);
    }
    buttons.add_child(Button::new("Back", |s| {
        s.pop_layer();
        show_wallets(s);
    }));

    let mut select = SelectView::<String>::new().on_submit(select_account);

    for account in &data.wallets[data.wallet_idx].accounts {
        let tag = format!("{}: {}", account.index, account.address);
        select.add_item_str(&tag)
    }

    let select = OnEventView::new(select).on_pre_event_inner(EventTrigger::mouse(), |s, e| {
        if let &Event::Mouse {
            event: MouseEvent::WheelUp,
            ..
        } = e
        {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        } else if let &Event::Mouse {
            event: MouseEvent::WheelDown,
            ..
        } = e
        {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        } else {
            None
        }
    });
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical().child(DummyView).child(
                LinearLayout::horizontal().child(
                    LinearLayout::vertical()
                        .child(buttons)
                        .child(DummyView)
                        .child(
                            Dialog::around(
                                select
                                    .with_name("accounts")
                                    .scrollable()
                                    .max_width(38)
                                    .max_height(5),
                            )
                            .padding_lrtb(1, 1, 0, 0)
                            .title("Accounts"),
                        ),
                ),
            ),
        )
        .title(wallet_name),
    );
}
