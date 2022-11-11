use super::process::process_send;
use crate::app::components::addressbook::ui::primary::show_addressbook;
use crate::app::components::inbox::ui::primary::show_inbox;
use crate::app::themes::get_subtitle_colour;
use crate::app::{clipboard::*, userdata::UserData};
use crate::crypto::conversions::raw_to_whole;
use crate::crypto::{address::validate_address, conversions::whole_to_raw};
use cursive::views::{
    Button, Checkbox, Dialog, DummyView, HideableView, LinearLayout, TextArea, TextView, ViewRef,
};
use cursive::{
    align::HAlign,
    traits::{Nameable, Resizable},
    utils::markup::StyledString,
    Cursive,
};

pub fn show_send(s: &mut Cursive, with_message: bool) {
    let mut address = String::from("");
    s.call_on_name("address", |view: &mut TextArea| {
        address = String::from(view.get_content());
    });
    let mut amount = String::from("");
    s.call_on_name("amount", |view: &mut TextArea| {
        amount = String::from(view.get_content());
    });

    s.pop_layer();

    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let account = &wallet.accounts[wallet.acc_idx];
    let balance = account.balance;
    let coin = data.coins[data.coin_idx].name.clone();
    let ticker = data.coins[data.coin_idx].ticker.clone();
    let multiplier = data.coins[data.coin_idx].multiplier.clone();

    let mut checkbox = Checkbox::new().on_change(show_send);
    if with_message {
        checkbox.check();
    }

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

    let sub_title_colour = get_subtitle_colour(data.coins[data.coin_idx].colour);

    let mut address_entry = TextArea::new();
    address_entry.set_cursor(address.len());
    let address_entry = address_entry
        .content(address)
        .with_name("address")
        .max_width(68);

    let mut form_content = LinearLayout::vertical()
        .child(TextView::new(StyledString::styled(
            "Recipient Address",
            sub_title_colour,
        )))
        .child(address_entry)
        .child(
            LinearLayout::horizontal()
                .child(Button::new("Paste", |s| {
                    let mut address: ViewRef<TextArea> = s.find_name("address").unwrap();
                    address.set_content(paste_clip(s));
                }))
                .child(Button::new("Address book", show_addressbook)),
        )
        .child(DummyView);
    let title_content;

    let bal = balance.to_string();
    let multi = multiplier.clone();

    if with_message {
        title_content = String::from("Send message");
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
    } else {
        title_content = format!("Send {}", coin);
        form_content.add_child(
            LinearLayout::horizontal()
                .child(TextView::new(StyledString::styled(
                    "Amount",
                    sub_title_colour,
                )))
                .child(DummyView)
                .child(Button::new("All", move |s| {
                    s.call_on_name("amount", |view: &mut TextArea| {
                        view.set_content(raw_to_whole(&bal, &multi))
                    })
                    .unwrap();
                })),
        );
    }

    let mut amount_entry = TextArea::new();
    amount_entry.set_cursor(amount.len());
    let amount_entry = amount_entry.content(amount).with_name("amount");
    form_content.add_child(amount_entry);
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
                address = address.trim().to_string();
                if !validate_address(&address) {
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
            .child(Button::new("Back", show_inbox))
            .child(DummyView)
            .child(DummyView)
            .child(DummyView)
            .child(TextView::new("With message "))
            .child(checkbox),
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
