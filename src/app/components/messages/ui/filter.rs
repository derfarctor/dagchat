use super::super::structs::Filter;
use super::primary::show_messages;
use crate::app::{constants::colours::OFF_WHITE, userdata::UserData};
use cursive::align::HAlign;
use cursive::utils::markup::StyledString;
use cursive::views::{Dialog, DummyView, LinearLayout, RadioGroup, TextView};
use cursive::Cursive;

pub fn show_filter(s: &mut Cursive, filter: Filter) {
    let mut message_dir: RadioGroup<u8> = RadioGroup::new();
    let mut message_amount: RadioGroup<u8> = RadioGroup::new();

    let data = &mut s.user_data::<UserData>().unwrap();

    let content = LinearLayout::vertical()
        .child(DummyView)
        .child(TextView::new(StyledString::styled(
            "Message type",
            OFF_WHITE,
        )))
        .child(
            LinearLayout::horizontal()
                .child(message_dir.button(0, "Both").selected())
                .child(DummyView)
                .child(message_dir.button(1, "Sent"))
                .child(DummyView)
                .child(message_dir.button(2, "Received")),
        )
        .child(DummyView)
        .child(TextView::new(StyledString::styled(
            "Message amount",
            OFF_WHITE,
        )))
        .child(
            LinearLayout::horizontal()
                .child(message_amount.button(0, "Both").selected())
                .child(DummyView)
                .child(message_amount.button(1, "1 RAW"))
                .child(DummyView)
                .child(message_amount.button(2, format!("Custom {}", data.coin.ticker.trim()))),
        );
    s.add_layer(
        Dialog::around(content)
            .h_align(HAlign::Center)
            .button("Apply", move |s| {
                let mut filter = filter.clone();
                let dir = message_dir.selection();
                let amount = message_amount.selection();
                if *dir == 0 {
                    filter.outgoing = true;
                    filter.outgoing = true;
                } else if *dir == 1 {
                    filter.outgoing = true;
                    filter.incoming = false;
                } else if *dir == 2 {
                    filter.outgoing = false;
                    filter.incoming = true;
                }
                if *amount == 0 {
                    filter.eq_1_raw = true;
                    filter.gt_1_raw = true;
                } else if *amount == 1 {
                    filter.eq_1_raw = true;
                    filter.gt_1_raw = false;
                } else if *amount == 2 {
                    filter.eq_1_raw = false;
                    filter.gt_1_raw = true;
                }
                s.pop_layer();
                s.pop_layer();
                show_messages(s, filter);
            })
            .title("Filter setup"),
    );
}
