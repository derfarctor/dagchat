use arboard::Clipboard;
use cursive::align::HAlign;
use cursive::theme::{BaseColor, BorderStyle, Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ProgressBar, RadioGroup, SelectView,
    TextArea, TextView,
};
use cursive::Cursive;
use rand::RngCore;

pub mod defaults;

// Dagchat util
mod dcutil;
use dcutil::*;

pub struct Data {
    pub account: Account,
    pub prefix: String,
    pub coin: String,
    pub ticker: String,
    pub node_url: String,
    pub colour: Color,
}

pub struct Account {
    pub entropy: [u8; 32],
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub address: String,
    pub balance: u128,
    pub receivables: Vec<Receivable>,
}

impl Default for Account {
    fn default() -> Account {
        Account {
            entropy: [0u8; 32],
            private_key: [0u8; 32],
            public_key: [0u8; 32],
            address: String::from(""),
            balance: 0,
            receivables: Vec::new(),
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Data {
            account: Default::default(),
            coin: String::from("nano"),
            prefix: String::from("nano_"),
            ticker: String::from("Ӿ"),
            node_url: String::from("https://app.natrium.io/api"),
            colour: L_BLUE,
        }
    }
}

const VERSION: &str = "alpha v1.0.0";

const L_BLUE: Color = Color::Rgb(62, 138, 227);
const M_BLUE: Color = Color::Rgb(0, 106, 255);
const D_BLUE: Color = Color::Rgb(12, 37, 125);

const YELLOW: Color = Color::Light(BaseColor::Yellow);
const OFF_WHITE: Color = Color::Rgb(245, 245, 247);

fn main() {
    let mut siv = cursive::default();
    siv.set_window_title(format!("dagchat {}", VERSION));
    let data = Data::new();
    siv.set_user_data(data);
    set_theme(&mut siv, "nano", false);

    let mut theme_group: RadioGroup<bool> = RadioGroup::new();

    let mut coin_group: RadioGroup<String> = RadioGroup::new();

    let radios = LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
                .child(coin_group.button("nano".to_string(), "nano").selected())
                .child(coin_group.button("banano".to_string(), "banano")),
        )
        .child(DummyView)
        .child(
            LinearLayout::vertical()
                .child(theme_group.button(false, "Modest").selected())
                .child(theme_group.button(true, "Vibrant")),
        );

    let button = Button::new_raw("Start", move |s| {
        let coin = coin_group.selection();
        let vibrant = theme_group.selection();
        set_theme(s, &*coin, *vibrant);
        if *coin == "banano" {
            s.with_user_data(|data: &mut Data| {
                data.coin = String::from("banano");
                data.prefix = String::from("ban_");
                data.ticker = String::from(" BAN");
                data.node_url = String::from("https://kaliumapi.appditto.com/api");
                data.colour = YELLOW;
            });
        }
        show_start(s)
    });

    siv.add_layer(
        Dialog::new()
            .content(
                LinearLayout::vertical()
                    .child(DummyView)
                    .child(button)
                    .child(DummyView)
                    .child(radios),
            )
            .title("Ӿdagchat v.1.0.0")
            .h_align(HAlign::Center),
    );
    siv.run();
}

fn show_start(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<Data>().unwrap();
    let coin = &data.coin;
    let colour = data.colour;
    let content = format!("Choose a way to import your {} wallet", coin);
    s.add_layer(
        Dialog::text(StyledString::styled(content, colour))
            .title("Import account")
            .h_align(HAlign::Center)
            .button("Mnemonic", |s| get_mnemonic(s))
            .button("Seed", |s| get_seed(s))
            .button("New", |s| new_account(s)),
    );
}

fn show_send(s: &mut Cursive, with_message: bool) {
    s.pop_layer();

    let data = &s.user_data::<Data>().unwrap();
    let balance = data.account.balance;
    let ticker = data.ticker.clone();
    let coin = data.coin.clone();

    if balance == 0 {
        let address = data.account.address.clone();
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
                .button("Copy Address", move |_| {
                    let mut clipboard = Clipboard::new().unwrap();
                    clipboard.set_text(address.clone()).unwrap();
                })
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
                    let raw_opt = whole_to_raw(amount);
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
                        if raw == 0 {
                            s.add_layer(Dialog::info(format!(
                                "You must provide an amount of {} to send!",
                                coin
                            )));
                            return;
                        }
                    }
                }
                process_send(s, raw, address, message);
            }))
            .child(Button::new("Cancel", |s| show_inbox(s))),
    );
    s.add_layer(
        Dialog::around(form_content)
            .title(title_content)
            .padding_lrtb(1, 1, 1, 0),
    );
}

fn go_back(s: &mut Cursive) {
    s.pop_layer();
}

fn get_subtitle_colour(s: &mut Cursive) -> Color {
    let data = &s.user_data::<Data>().unwrap();
    let sub_title_colour;
    if data.colour == YELLOW {
        sub_title_colour = OFF_WHITE;
    } else {
        sub_title_colour = data.colour;
    }
    sub_title_colour
}

fn process_send(s: &mut Cursive, raw: u128, address: String, message: String) {
    let ticks = 1000;
    let cb = s.cb_sink().clone();
    let data = &mut s.user_data::<Data>().unwrap();
    let node_url = data.node_url.clone();
    let private_key_bytes = data.account.private_key;
    let prefix = data.prefix.clone();
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
                    let data = &mut s.user_data::<Data>().unwrap();
                    data.account.balance -= raw;
                    show_sent(s, with_message);
                }))
                .unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}

fn process_receive(s: &mut Cursive, idx: usize) {
    let data = &mut s.user_data::<Data>().unwrap();
    let private_key = data.account.private_key;
    let send_block_hash = data.account.receivables[idx].hash.clone();
    let amount = data.account.receivables[idx].amount;
    let address = data.account.address.clone();
    let prefix = data.prefix.clone();
    let node_url = data.node_url.clone();
    let ticks = 1000;
    let cb = s.cb_sink().clone();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                counter.tick(100);
                receive_block(
                    &private_key,
                    &send_block_hash,
                    amount,
                    &address,
                    &node_url,
                    &prefix,
                    &counter,
                );
                cb.send(Box::new(move |s| {
                    let mut select = s.find_name::<SelectView<String>>("select").unwrap();
                    select.remove_item(idx);
                    let mut balance = s.find_name::<TextView>("balance").unwrap();
                    let data = &mut s.user_data::<Data>().unwrap();
                    data.account.receivables.remove(idx);
                    data.account.balance += amount;
                    let bal = display_to_dp(data.account.balance, 5, &data.ticker);
                    let bal_text = format!("Balance: {}", bal);
                    balance.set_content(StyledString::styled(bal_text, data.colour));
                    s.pop_layer();
                    s.pop_layer();
                }))
                .unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}

fn alpha_info(s: &mut Cursive) {
    s.pop_layer();
    let mut info = StyledString::plain("Information for alpha testers:\n");
    info.append(StyledString::styled("This is the inbox. Messages sent to you with 1 raw have already been identified as messages, but messages sent with an arbitrary amount will not yet have been detected. Select 'Find messages' from the buttons on the right to scan your list of receivables and identify these.", OFF_WHITE));
    s.add_layer(
        Dialog::around(TextView::new(info))
            .button("Go to inbox", |s| load_receivables(s, true))
            .max_width(60),
    );
}

fn show_sent(s: &mut Cursive, with_message: bool) {
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

fn load_receivables(s: &mut Cursive, initial: bool) {
    let ticks = 1000;

    let cb = s.cb_sink().clone();

    let data = &s.user_data::<Data>().unwrap();
    let node_url = data.node_url.clone();
    let target_address = data.account.address.clone();
    s.pop_layer();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                let account_info = get_account_info(&target_address, &node_url);
                let mut balance: u128 = 0;
                if account_info.is_some() {
                    balance = get_balance(&account_info.unwrap());
                }
                counter.tick(50);
                let mut receivables = find_incoming(&target_address, &node_url);
                counter.tick(150);
                if !receivables.is_empty() {
                    let x = 800usize / receivables.len();
                    for receivable in &mut receivables {
                        if initial {
                            if receivable.amount == 1 {
                                receivable.message = has_message(&receivable.hash, &node_url);
                            }
                        } else {
                            if receivable.amount != 1 {
                                receivable.message = has_message(&receivable.hash, &node_url);
                            }
                        }
                        counter.tick(x);
                    }
                }
                cb.send(Box::new(move |s| {
                    let data = &mut s.user_data::<Data>().unwrap();
                    if !initial {
                        for rx in receivables {
                            let existing_idx = data
                                .account
                                .receivables
                                .iter()
                                .position(|r| &r.hash == &rx.hash);
                            if rx.message.is_none() {
                                continue;
                            };
                            if existing_idx.is_none() {
                                data.account.receivables.push(rx);
                            } else {
                                data.account.receivables[existing_idx.unwrap()] = rx;
                            }
                        }
                    } else {
                        data.account.receivables = receivables;
                    }
                    data.account.balance = balance;
                    show_inbox(s);
                }))
                .unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}

fn show_change_rep(s: &mut Cursive) {
    s.pop_layer();
    let data = &mut s.user_data::<Data>().unwrap();
    let private_key = data.account.private_key;
    let prefix = data.prefix.clone();
    let node_url = data.node_url.clone();
    let address = data.account.address.clone();
    let coin = data.coin.clone();
    let sub_title_colour = get_subtitle_colour(s);
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
            .child(DummyView)
                .child(TextView::new(StyledString::styled(
                    "Representative address",
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
                .child(DummyView)
                .child(LinearLayout::horizontal()
                .child(Button::new("Change", move |s| {
                    let mut rep_address = String::from("");
                    s.call_on_name("address", |view: &mut TextArea| {
                        rep_address = String::from(view.get_content());
                    });
                    if rep_address.is_empty() {
                        s.add_layer(Dialog::info(
                            "You must provide an address to change representative to!",
                        ));
                        return;
                    }
                    let valid = validate_address(&rep_address);
                    if !valid {
                        s.add_layer(Dialog::info("The representative's address is invalid."));
                        return;
                    }
                    let account_info_opt = get_account_info(&address, &node_url);
                    if account_info_opt.is_none() {
                        s.add_layer(Dialog::info(format!("You can't change representatives until you open your account by receiving some {}.", coin)));
                        return;
                    }
                    let account_info = account_info_opt.unwrap();
                    change_rep(&private_key, account_info, &rep_address, &node_url, &prefix);
                    s.pop_layer();
                    show_inbox(s);
                    s.add_layer(Dialog::info("Successfully changed representative!"));
                }))
                .child(Button::new("Cancel", |s| show_inbox(s)))),
        )
        .title("Change representative"),
    );
}

fn copy_to_clip(s: &mut Cursive, string: String) {
    let mut clipboard = Clipboard::new().unwrap();
    let copied = clipboard.set_text(string.clone());
    if copied.is_err() {
        s.add_layer(Dialog::info("Error copying to clipboard."));
    } else {
        let mut content = StyledString::styled(format!("{}\n", string), OFF_WHITE);
        content.append(StyledString::plain("was successfully copied to your clipboard."));
        s.add_layer(
            Dialog::around(TextView::new(content))
                .dismiss_button("Back")
                .max_width(80),
        );
    }
}

fn show_inbox(s: &mut Cursive) {
    s.set_autorefresh(false);
    s.pop_layer();
    let data: Data = s.take_user_data().unwrap();
    let address = data.account.address.clone();
    let send_label = format!("Send {}", data.coin);
    let buttons = LinearLayout::vertical()
        .child(Button::new("Refresh", |s| load_receivables(s, true)))
        .child(DummyView)
        .child(Button::new(send_label, |s| show_send(s, false)))
        .child(Button::new("Send message", |s| show_send(s, true)))
        .child(DummyView)
        .child(Button::new("Find messages", |s| load_receivables(s, false)))
        .child(Button::new("Copy address", move |s| copy_to_clip(s, address.clone())))
        .child(Button::new("Change rep", |s| show_change_rep(s)))
        .child(DummyView)
        .child(Button::new("Quit", |s| s.quit()));

    let select = SelectView::<String>::new()
        .on_submit(show_message_info)
        .with_name("select")
        .scrollable()
        .max_height(6);

    let bal = display_to_dp(data.account.balance, 5, &data.ticker);
    let bal_text = format!("Balance: {}", bal);
    let bal_content =
        TextView::new(StyledString::styled(bal_text, data.colour)).with_name("balance");
    s.add_layer(
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
                                .title("Receivables"),
                        ),
                )
                .child(DummyView)
                .child(DummyView)
                .child(LinearLayout::vertical().child(DummyView).child(buttons)),
        )
        .title(format!("dagchat {}", VERSION)),
    );

    for receivable in &data.account.receivables {
        let mut tag;
        if receivable.amount == 1 && receivable.message.is_some() {
            tag = String::from("Message");
        } else {
            tag = display_to_dp(receivable.amount, 5, &data.ticker);
            if receivable.message.is_some() {
                tag = format!("{} + Msg", tag);
            }
        }
        let addr = receivable.source.get(0..11).unwrap();
        tag = format!("{} > {}", addr, tag);
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(&tag)
        });
    }
    s.set_user_data(data);
}

fn show_message_info(s: &mut Cursive, _name: &str) {
    let select = s.find_name::<SelectView<String>>("select").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No receivable selected.")),
        Some(focus) => {
            let data = &mut s.user_data::<Data>().unwrap();
            let receivable = &mut data.account.receivables[focus];
            let private_key = &data.account.private_key;
            let node_url = &data.node_url;
            let plaintext: String;

            let mut content = LinearLayout::vertical();
            let mut title = format!("{} Receivable", &data.ticker);
            let mut receive_label = String::from("");
            if receivable.message.is_some() {
                receive_label = String::from(" and mark read");
                title = String::from("Message");
                let mut message = receivable.message.as_mut().unwrap();
                if message.plaintext.is_empty() {
                    // Potential feature: Confirm option with message length in chars (estimated)
                    // removes ability for attacks such as extremely long messages although probably
                    // not an issue. Harder to send a long message than read.
                    let target = &message.head.as_mut().unwrap().contents.account;
                    let root_hash = &message.root_hash;
                    let blocks = message.blocks;
                    // Potential feature: Add loading screen + process_message()
                    // time taken to load a (long) message can be noticeable if node
                    // is under load.
                    plaintext = read_message(private_key, target, root_hash, blocks, node_url);
                    message.plaintext = plaintext.clone();
                } else {
                    plaintext = message.plaintext.clone();
                }
                content.add_child(
                    TextView::new(plaintext)
                        .scrollable()
                        .max_width(80)
                        .max_height(6),
                );
                content.add_child(DummyView);
            }
            let colour = data.colour;
            if !(receivable.amount == 1 && receivable.message.is_some()) {
                receive_label = format!("Receive{}", receive_label);
                let amount = display_to_dp(receivable.amount, 5, &data.ticker);
                content.add_child(TextView::new(StyledString::styled("Amount", colour)));
                content.add_child(TextView::new(StyledString::styled(amount, OFF_WHITE)));
                content.add_child(DummyView);
            } else {
                receive_label = String::from("Mark read");
            }
            let sender = receivable.source.clone();
            content.add_child(TextView::new(StyledString::styled("From", colour)));
            content
                .add_child(TextView::new(StyledString::styled(sender, OFF_WHITE)).fixed_width(65));

            s.add_layer(
                Dialog::around(content)
                    .button(receive_label, move |s| {
                        process_receive(s, focus);
                    })
                    .button("Back", |s| go_back(s))
                    .title(title),
            );
        }
    }
}

fn get_mnemonic(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .title("Enter your 24 word mnemonic")
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    .on_submit(set_mnemonic)
                    .with_name("mnemonic")
                    .fixed_width(29),
            )
            .h_align(HAlign::Center)
            .button("Done", |s| {
                let mnemonic = s
                    .call_on_name("mnemonic", |view: &mut EditView| view.get_content())
                    .unwrap();

                set_mnemonic(s, &mnemonic);
            })
            .button("Paste", |s| {
                s.call_on_name("mnemonic", |view: &mut EditView| {
                    let mut clipboard = Clipboard::new().unwrap();
                    let clip = clipboard
                        .get_text()
                        .unwrap_or_else(|_| String::from("Failed to read clipboard."));
                    view.set_content(clip);
                })
                .unwrap();
            })
            .button("Back", |s| {
                show_start(s);
            }),
    );
}

fn get_seed(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .title("Enter your hex seed")
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    .on_submit(set_mnemonic)
                    .with_name("seed")
                    .fixed_width(29),
            )
            .h_align(HAlign::Center)
            .button("Done", |s| {
                let seed = s
                    .call_on_name("seed", |view: &mut EditView| view.get_content())
                    .unwrap();
                if seed.len() != 64 {
                    s.add_layer(Dialog::info("Seed was invalid: not 64 characters long."));
                    return;
                }
                let bytes_opt = hex::decode(&*seed);
                if bytes_opt.is_err() {
                    s.add_layer(Dialog::info("Seed was invalid: failed to decode hex."));
                    return;
                }
                let bytes = bytes_opt.unwrap();
                set_seed(s, bytes);
            })
            .button("Paste", |s| {
                s.call_on_name("seed", |view: &mut EditView| {
                    let mut clipboard = Clipboard::new().unwrap();
                    let clip = clipboard
                        .get_text()
                        .unwrap_or_else(|_| String::from("Failed to read clipboard."));
                    view.set_content(clip);
                })
                .unwrap();
            })
            .button("Back", |s| {
                show_start(s);
            }),
    );
}

fn set_seed(s: &mut Cursive, seed: Vec<u8>) {
    let mut entropy_bytes = [0u8; 32];
    entropy_bytes.copy_from_slice(seed.as_slice());
    s.pop_layer();
    setup_account(s, entropy_bytes);
    let content = "Successfully imported account.";
    s.add_layer(Dialog::around(TextView::new(content)).button("Begin", |s| alpha_info(s)));
}

fn new_account(s: &mut Cursive) {
    let mut csprng = rand::thread_rng();
    let mut entropy_bytes = [0u8; 32];
    csprng.fill_bytes(&mut entropy_bytes);
    let entropy_hex = hex::encode(entropy_bytes);
    setup_account(s, entropy_bytes);
    s.pop_layer();
    let mut content = StyledString::plain("Successfully generated new account with seed: ");
    content.append(StyledString::styled(&entropy_hex, OFF_WHITE));
    s.add_layer(Dialog::around(
        TextView::new(content))
            .button("Copy seed", move |s| { copy_to_clip(s, entropy_hex.clone())})
            .button("Begin", |s| alpha_info(s)),
    );
}

fn setup_account(s: &mut Cursive, entropy_bytes: [u8; 32]) {
    let data = &mut s.user_data::<Data>().unwrap();
    let private_key_bytes = get_private_key(&entropy_bytes);
    let private_key = ed25519_dalek::SecretKey::from_bytes(&private_key_bytes).unwrap();
    let public_key = ed25519_dalek::PublicKey::from(&private_key);
    let public_key_bytes = public_key.to_bytes();
    let address = get_address(&public_key_bytes, &data.prefix);
    data.account.entropy = entropy_bytes;
    data.account.private_key = private_key_bytes;
    data.account.public_key = public_key_bytes;
    data.account.address = address;
}

fn set_mnemonic(s: &mut Cursive, mnemonic: &str) {
    let entropy = validate_mnemonic(mnemonic);
    let content;
    s.pop_layer();
    if !mnemonic.is_empty() && entropy.is_some() {
        let entropy_bytes = entropy.unwrap();
        setup_account(s, entropy_bytes);
        content = "Successfully imported account.";
        s.add_layer(Dialog::around(TextView::new(content)).button("Begin", |s| alpha_info(s)));
    } else {
        content = "The mnemonic you entered was not valid.";
        s.add_layer(Dialog::around(TextView::new(content)).button("Back", |s| get_mnemonic(s)));
    }
}

fn set_theme(s: &mut Cursive, style: &str, vibrant: bool) {
    let mut theme = s.current_theme().clone();
    if style == "nano" {
        theme = get_nano_theme(theme, vibrant);
    } else {
        theme = get_banano_theme(theme, vibrant);
    }
    s.set_theme(theme);
}

fn get_banano_theme(mut base: Theme, v: bool) -> Theme {
    if v {
        base.shadow = true;
        base.palette[PaletteColor::Background] = YELLOW;
    } else {
        base.palette[PaletteColor::Background] = Color::Rgb(25, 25, 27);
    }
    base.palette[PaletteColor::View] = Color::Rgb(34, 34, 42);
    base.palette[PaletteColor::Primary] = YELLOW;
    base.palette[PaletteColor::Secondary] = YELLOW;
    base.palette[PaletteColor::Tertiary] = OFF_WHITE;
    base.palette[PaletteColor::TitlePrimary] = OFF_WHITE;
    base.palette[PaletteColor::TitleSecondary] = YELLOW;
    base.palette[PaletteColor::Highlight] = Color::Dark(BaseColor::Yellow);
    base.palette[PaletteColor::HighlightInactive] = YELLOW;
    base.palette[PaletteColor::Shadow] = Color::Dark(BaseColor::Yellow);
    base
}

fn get_nano_theme(mut base: Theme, v: bool) -> Theme {
    if v {
        base.shadow = true;
        base.palette[PaletteColor::Background] = L_BLUE;
        base.palette[PaletteColor::Shadow] = D_BLUE;
    } else {
        base.shadow = false;
        base.palette[PaletteColor::Background] = Color::Rgb(25, 25, 27);
    }
    base.borders = BorderStyle::Simple;
    base.palette[PaletteColor::View] = Color::Rgb(34, 34, 42);
    base.palette[PaletteColor::Primary] = OFF_WHITE;
    base.palette[PaletteColor::Secondary] = OFF_WHITE;
    base.palette[PaletteColor::Tertiary] = M_BLUE;
    base.palette[PaletteColor::TitlePrimary] = OFF_WHITE;
    base.palette[PaletteColor::TitleSecondary] = YELLOW;
    base.palette[PaletteColor::Highlight] = D_BLUE;
    base.palette[PaletteColor::HighlightInactive] = L_BLUE;
    base
}
