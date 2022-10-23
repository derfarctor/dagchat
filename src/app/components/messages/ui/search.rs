use super::super::structs::Filter;
use super::primary::show_messages;
use crate::app::{clipboard::paste_clip, constants::colours::OFF_WHITE, helpers::go_back};
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Button, Dialog, DummyView, LinearLayout, TextArea, TextView};
use cursive::{align::HAlign, utils::markup::StyledString, Cursive};

pub fn show_search(s: &mut Cursive, filter: Filter) {
    let content = LinearLayout::vertical()
        .child(DummyView)
        .child(TextView::new(StyledString::styled(
            "Search term or address",
            OFF_WHITE,
        )))
        .child(TextArea::new().with_name("search").max_width(66))
        .child(LinearLayout::horizontal().child(Button::new("Paste", |s| {
            s.call_on_name("search", |view: &mut TextArea| {
                view.set_content(paste_clip());
            })
            .unwrap();
        })));
    s.add_layer(
        Dialog::around(content)
            .h_align(HAlign::Center)
            .button("Search", move |s| {
                let mut filter = filter.clone();
                s.call_on_name("search", |view: &mut TextArea| {
                    if view.get_content().trim().is_empty() {
                        filter.search_term = None;
                    } else {
                        filter.search_term = Some(String::from(view.get_content()));
                    }
                });
                s.pop_layer();
                s.pop_layer();
                show_messages(s, filter);
            })
            .button("Back", go_back)
            .title("Search"),
    )
}
