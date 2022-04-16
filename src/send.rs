use super::*;

pub fn show_send(s: &mut Cursive, with_message: bool) {
    s.pop_layer();

    let data = &s.user_data::<UserData>().unwrap();
    let balance = data.accounts[data.acc_idx].balance;
    let coin = data.coin.name.clone();
    let ticker = data.coin.ticker.clone();
    let multiplier = data.coin.multiplier.clone();

    if balance == 0 {
        let address = data.accounts[data.acc_idx].address.clone();
        let no_balance_message;
        if with_message {
            no_balance_message = format!("To send a message with dagchat you need a balance of at least 1 raw - a tiny fraction of a coin. One faucet claim will last you a lifetime. Your address is: {}", address);
        } else {
            no_balance_message = format!("You don't have any {} in your wallet to send.", coin);
        }
        s.add_layer(
            Dialog::around(TextView::new(no_balance_message))
                .h_align(HAlign::Center)
                .button("Back", |s| show_inbox(s))
                .button("Copy Address", move |s| copy_to_clip(s, address.clone()))
                .max_width(75),
        );
        return;
    }

    let sub_title_colour = get_subtitle_colour(s);

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
                        let mut clipboard = Clipboard::new().unwrap();
                        let clip = clipboard
                            .get_text()
                            .unwrap_or_else(|_| String::from("Failed to read clipboard."));
                        view.set_content(clip);
                    })
                    .unwrap();
                }))
                .child(Button::new("Address book", |s| {
                    s.add_layer(Dialog::info("Coming soon..."));
                })),
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
            format!("Optional {}", ticker),
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
                    let content;
                    if with_message {
                        content =
                            String::from("You must provide an address to send the message to!");
                    } else {
                        content = format!("You must provide an address to send {} to!", coin);
                    }
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
                        let content;
                        if with_message {
                            content = "The optional amount was invalid.";
                        } else {
                            content = "The amount was invalid.";
                        }
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
                } else {
                    if message.is_empty() {
                        // The user supplied no amount and it's not a message
                        s.add_layer(Dialog::info(format!(
                            "You must provide an amount of {} to send!",
                            coin
                        )));
                        return;
                    }
                }
                process_send(s, raw, address, message);
            }))
            .child(Button::new("Back", |s| show_inbox(s))),
    );
    s.add_layer(
        Dialog::around(form_content)
            .title(title_content)
            .padding_lrtb(1, 1, 1, 0),
    );
}

pub fn show_sent(s: &mut Cursive, with_message: bool) {
    s.set_autorefresh(false);
    s.pop_layer();
    let content;
    if with_message {
        content = "Message sent successfully!";
    } else {
        content = "Sent successfully!";
    }
    s.add_layer(Dialog::text(content).button("Back", |s| show_inbox(s)));
}

fn process_send(s: &mut Cursive, raw: u128, address: String, message: String) {
    let ticks = 1000;
    let cb = s.cb_sink().clone();
    let data = &mut s.user_data::<UserData>().unwrap();
    let node_url = data.coin.node_url.clone();
    let private_key_bytes = data.accounts[data.acc_idx].private_key;
    let prefix = data.coin.prefix.clone();
    s.pop_layer();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                let with_message = !message.is_empty();
                if !with_message {
                    send(
                        &private_key_bytes,
                        address,
                        raw,
                        &node_url,
                        &prefix,
                        &counter,
                    );
                } else {
                    send_message(
                        &private_key_bytes,
                        address,
                        raw,
                        message,
                        &node_url,
                        &prefix,
                        &counter,
                    );
                }
                cb.send(Box::new(move |s| {
                    let data = &mut s.user_data::<UserData>().unwrap();
                    data.accounts[data.acc_idx].balance -= raw;
                    show_sent(s, with_message);
                }))
                .unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}
