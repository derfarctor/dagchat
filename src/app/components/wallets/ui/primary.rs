use super::{
    add::{add_wallet, new_wallet_name},
    backup::backup_wallet,
    remove::remove_wallet,
    select::select_wallet,
};
use crate::app::components::title::ui::primary::show_title;
use crate::app::userdata::UserData;
use cursive::event::{Event, EventResult, EventTrigger, MouseEvent};
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::views::{Button, Dialog, DummyView, LinearLayout, OnEventView, SelectView};
use cursive::Cursive;

pub fn show_wallets(s: &mut Cursive) {
    s.pop_layer();
    // Need to add change password button
    let buttons = LinearLayout::vertical()
        .child(Button::new("Import", add_wallet))
        .child(Button::new("Create", new_wallet_name))
        .child(DummyView)
        .child(Button::new("Backup", backup_wallet))
        .child(Button::new("Delete", remove_wallet))
        .child(DummyView)
        .child(Button::new("Back", |s| {
            s.pop_layer();
            show_title(s);
        }))
        .child(DummyView);

    let mut select = SelectView::<String>::new().on_submit(select_wallet);

    let mut i = 1;
    let data = &s.user_data::<UserData>().unwrap();
    for wallet in &data.wallets {
        let tag = format!("{}. {}", i, wallet.name);
        select.add_item_str(&tag);
        i += 1;
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
                LinearLayout::horizontal()
                    .child(
                        Dialog::around(select.with_name("wallets").scrollable().max_height(6))
                            .padding_lrtb(1, 1, 0, 0)
                            .title("Wallets"),
                    )
                    .child(DummyView)
                    .child(DummyView)
                    .child(buttons),
            ),
        )
        .title("Select a Wallet"),
    );
}
