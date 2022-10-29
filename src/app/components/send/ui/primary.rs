use super::process::process_send;
use crate::app::components::addressbook::ui::primary::show_addressbook;
use crate::app::components::inbox::ui::primary::show_inbox;
use crate::app::themes::get_subtitle_colour;
use crate::app::{clipboard::*, userdata::UserData};
use crate::crypto::{address::validate_address, conversions::whole_to_raw};
use cursive::views::{Button, Dialog, DummyView, HideableView, LinearLayout, TextArea, TextView};
use cursive::{
    align::HAlign,
    traits::{Nameable, Resizable},
    utils::markup::StyledString,
    Cursive,
};

pub fn show_send(s: &mut Cursive, with_message: bool) {
    s.pop_layer();

    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let account = &wallet.accounts[wallet.acc_idx];
    let balance = account.balance;
    let coin = data.coin.name.clone();
    let ticker = data.coin.ticker.clone();
    let multiplier = data.coin.multiplier.clone();

    if balance == 0 {
        let address = account.address.clone();
        let no_balance_message = if with_message {
            String::from("To send a message with dagchat you need a balance of at least 1 raw - a tiny fraction of a coin. One faucet claim will last you a lifetime.")
        } else {
            format!("You don't have any {} in your wallet to send.", coin)
        };
        s.add_layer(
            Dialog::around(TextView::new(no_balance_message))
                .h_align(HAlign::Center)
                .button("Back", show_inbox)
                .button("Copy Address", move |s| copy_to_clip(s, address.clone()))
                .max_width(75),
        );
        return;
    }

    let sub_title_colour = get_subtitle_colour(data.coin.colour);

    let mut form_content = LinearLayout::vertical()
        .child(TextView::new(StyledString::styled(
            "Recipient Address",
            sub_title_colour,
        )))
        .child(TextArea::new().with_name("address").max_width(66))
        .child(
            LinearLayout::horizontal()
                .child(Button::new("Paste", |s| {
                    s.call_on_name("address", |view: &mut TextArea| {
                        view.set_content(paste_clip());
                    })
                    .unwrap();
                }))
                .child(Button::new("Address book", show_addressbook)),
        )
        .child(DummyView);
    let title_content;
    if with_message {
        title_content = String::from("Send a message");
        form_content.add_child(TextView::new(StyledString::styled(
            "Message Content",
            sub_title_colour,
        )));
        form_content.add_child(TextArea::new().with_name("message").max_width(80));
        form_content.add_child(DummyView);
        form_content.add_child(TextView::new(StyledString::styled(
            format!("Optional {}", ticker.trim()),
            sub_title_colour,
        )));
        form_content.add_child(TextArea::new().with_name("amount"));
    } else {
        title_content = format!("Send {}", coin);
        form_content.add_child(TextView::new(StyledString::styled(
            "Amount",
            sub_title_colour,
        )));
        form_content.add_child(TextArea::new().with_name("amount"));
    }
    form_content.add_child(DummyView);
    form_content.add_child(
        LinearLayout::horizontal()
            .child(Button::new("Send", move |s| {
                let mut address = String::from("");
                let mut message = String::from("");
                let mut amount = String::from("");
                s.call_on_name("address", |view: &mut TextArea| {
                    address = String::from(view.get_content());
                })
                .unwrap();
                if with_message {
                    s.call_on_name("message", |view: &mut TextArea| {
                        message = String::from(view.get_content());
                    })
                    .unwrap();
                    if message.trim().is_empty() {
                        s.add_layer(Dialog::info(
                            "You must provide message content to send a message!",
                        ));
                        return;
                    }
                }
                s.call_on_name("amount", |view: &mut TextArea| {
                    amount = String::from(view.get_content());
                })
                .unwrap();
                if address.is_empty() {
                    let content = if with_message {
                        String::from("You must provide an address to send the message to!")
                    } else {
                        format!("You must provide an address to send {} to!", coin)
                    };
                    s.add_layer(Dialog::info(content));
                    return;
                }
                let valid = validate_address(&address);
                if !valid {
                    s.add_layer(Dialog::info("The recipient's address is invalid."));
                    return;
                }
                let mut raw: u128 = 0;
                if with_message {
                    raw = 1;
                }
                if !amount.is_empty() {
                    let raw_opt = whole_to_raw(amount, &multiplier);
                    if raw_opt.is_none() {
                        let content = if with_message {
                            "The optional amount was invalid."
                        } else {
                            "The amount was invalid."
                        };
                        s.add_layer(Dialog::info(content));
                        return;
                    }
                    raw = raw_opt.unwrap();
                    if raw > balance {
                        s.add_layer(Dialog::info(
                            "The amount you want to send is more than your account balance!",
                        ));
                        return;
                    } else {
                        // The user supplied the amount 0
                        if raw == 0 {
                            s.add_layer(Dialog::info(format!(
                                "You must provide an amount of {} to send!",
                                coin
                            )));
                            return;
                        }
                    }
                } else if message.is_empty() {
                    // The user supplied no amount and it's not a message
                    s.add_layer(Dialog::info(format!(
                        "You must provide an amount of {} to send!",
                        coin
                    )));
                    return;
                }
                process_send(s, raw, address, message);
            }))
            .child(Button::new("Back", show_inbox)),
    );
    s.add_layer(
        HideableView::new(
            Dialog::around(form_content)
                .title(title_content)
                .padding_lrtb(1, 1, 1, 0),
        )
        .with_name("hideable"),
    );
}
