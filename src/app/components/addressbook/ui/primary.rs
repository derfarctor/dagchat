use super::{add::add_addressbook, remove::remove_addressbook, select::select_addressbook};
use crate::app::{constants::AUTHOR, userdata::UserData};
use cursive::event::{Event, EventResult, EventTrigger, MouseEvent};
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::views::{
    Button, Dialog, DummyView, HideableView, LinearLayout, OnEventView, SelectView,
};
use cursive::Cursive;

pub fn show_addressbook(s: &mut Cursive) {
    s.call_on_name("hideable", |view: &mut HideableView<Dialog>| {
        view.set_visible(false);
    })
    .unwrap();

    let buttons = LinearLayout::vertical()
        .child(DummyView)
        .child(Button::new("Add", add_addressbook))
        .child(Button::new("Remove", remove_addressbook))
        .child(DummyView)
        .child(Button::new("Back", |s| {
            s.call_on_name("hideable", |view: &mut HideableView<Dialog>| {
                view.set_visible(true);
            })
            .unwrap();
            s.pop_layer();
        }))
        .child(DummyView);

    let data = &s.user_data::<UserData>().unwrap();
    let mut form_content = LinearLayout::horizontal();

    let mut select = SelectView::<String>::new().on_submit(select_addressbook);

    let mut names: Vec<String> = data.addressbook.values().cloned().collect();
    names.sort();
    for name in names {
        if name != AUTHOR {
            select.add_item_str(name);
        }
    }
    select.add_item_str(AUTHOR);
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
    form_content.add_child(
        Dialog::around(
            select
                .with_name("addressbook")
                .scrollable()
                .max_width(28)
                .max_height(6),
        )
        .padding_lrtb(1, 1, 0, 0),
    );

    form_content.add_child(DummyView);

    form_content.add_child(DummyView);
    form_content.add_child(buttons);
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(form_content),
        )
        .title("Address Book"),
    );
}
