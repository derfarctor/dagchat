use super::super::structs::Filter;
use super::{filter::show_filter, search::show_search};
use crate::app::{constants::colours::OFF_WHITE, helpers::go_back, userdata::UserData};
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use cursive::traits::{Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, TextView};
use cursive::Cursive;

pub fn show_messages(s: &mut Cursive, mut filter: Filter) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let messages = &wallet.accounts[wallet.acc_idx].messages;
    if messages.is_err() {
        let err_msg = messages.as_ref().err().unwrap().clone();
        s.add_layer(Dialog::info(err_msg));
        return;
    } else if messages.as_ref().unwrap().is_empty() {
        s.add_layer(Dialog::info(
            "You haven't sent or received any messages yet with dagchat on this account!",
        ));
        return;
    }

    let mut output = StyledString::new();
    let mut search_term = String::from("");
    if filter.search_term.is_some() {
        search_term = filter.search_term.unwrap();
    };

    for message in messages.as_ref().unwrap().iter().rev() {
        if (message.outgoing && !filter.outgoing)
            || (!message.outgoing && !filter.incoming)
            || (message.amount == "1 RAW" && !filter.eq_1_raw)
            || (message.amount != "1 RAW" && !filter.gt_1_raw)
        {
            continue;
        }

        let datetime: DateTime<Local> = DateTime::from(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(message.timestamp as i64, 0),
            Utc,
        ));

        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        let a: &str;
        let b: &str;
        if message.outgoing {
            a = "Sent";
            b = "To:";
        } else {
            a = "Received";
            b = "From:";
        }
        let colour = data.coins[data.coin_idx].colour;
        let mut message_info = StyledString::styled(format!("{} at: ", a), OFF_WHITE);
        message_info.append(StyledString::styled(timestamp_str, colour));
        message_info.append(StyledString::styled(format!("\n{} ", b), OFF_WHITE));

        let mut source_parts: Vec<&str> = message.address.split('_').collect();
        let source_suffix = String::from('_') + source_parts.pop().unwrap();
        let source = if data.addressbook.contains_key(&source_suffix) {
            data.addressbook.get(&source_suffix).unwrap()
        } else {
            &message.address
        };

        message_info.append(StyledString::styled(source, colour));
        if !message.plaintext.is_empty() {
            message_info.append(StyledString::styled("\nMessage: ", OFF_WHITE));
            if message.amount != "1 RAW" {
                message_info.append(StyledString::styled(&message.plaintext, colour));
                message_info.append(StyledString::styled("\nAmount: ", OFF_WHITE));
                message_info.append(StyledString::styled(
                    format!("{}\n\n", message.amount),
                    colour,
                ));
            } else {
                message_info.append(StyledString::styled(
                    format!("{}\n\n", &message.plaintext),
                    colour,
                ))
            };
        } else {
            message_info.append(StyledString::styled("\nAmount: ", OFF_WHITE));
            message_info.append(StyledString::styled(
                format!("{}\n\n", message.amount),
                colour,
            ));
        }

        if !search_term.as_str().is_empty() {
            if message_info.source().contains(&search_term) {
                output.append(message_info);
            }
        } else {
            output.append(message_info);
        }
    }
    if search_term.is_empty() {
        filter.search_term = None;
    } else {
        filter.search_term = Some(search_term);
    }

    // Annoying reallocations due to having multiple closures
    // requiring filter. Need to look into how to solve although
    // minimal perfomance hit.
    let search_filter = filter.clone();
    let filter_copy = filter.clone();

    let mut content = LinearLayout::vertical()
        .child(
            LinearLayout::horizontal()
                .child(Button::new("Search", move |s| {
                    show_search(s, search_filter.clone())
                }))
                .child(DummyView)
                .child(Button::new("Filter", move |s| {
                    show_filter(s, filter_copy.clone())
                }))
                .child(DummyView)
                .child(Button::new("Back", go_back)),
        )
        .child(DummyView);
    if filter.search_term.is_some() {
        content.add_child(TextView::new(StyledString::styled(
            format!("Contains: {}", filter.search_term.unwrap()),
            OFF_WHITE,
        )))
    }
    if output.is_empty() {
        content.add_child(DummyView);
        content.add_child(TextView::new(StyledString::styled(
            "No messages found.",
            data.coins[data.coin_idx].colour,
        )));
    }
    content.add_child(
        TextView::new(output)
            .scrollable()
            .max_width(77)
            .max_height(12),
    );
    s.add_layer(Dialog::around(content).title("Message history"));
}
