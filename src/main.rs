use arboard::Clipboard;
use bigdecimal::BigDecimal;
use cursive::align::HAlign;
use cursive::theme::{BaseColor, BorderStyle, Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::views::{
    ScrollView,
    Button, Dialog, DummyView, EditView, LinearLayout, ProgressBar, RadioGroup, SelectView,
    TextArea, TextView,
};
use cursive::Cursive;
use std::str::FromStr;
use std::convert::TryFrom;

mod lib;

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
    pub receivables: Vec<lib::Receivable>,
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

const VERSION: &str = "1.0.0";

const L_BLUE: Color = Color::Rgb(62, 138, 227);
const M_BLUE: Color = Color::Rgb(0, 106, 255);
const D_BLUE: Color = Color::Rgb(12, 37, 125);

const YELLOW: Color = Color::Light(BaseColor::Yellow);
const OFF_WHITE: Color = Color::Rgb(245, 245, 247);

fn main() {
    let mut siv = cursive::default();
    siv.set_window_title(format!("dagchat v{}", VERSION));
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
                data.ticker = String::from("BAN");
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
            .button("mnemonic", |s| get_mnemonic(s)),
    );
}

fn show_send(s: &mut Cursive) {
    s.pop_layer();
    let mut empty = true;
    let mut address = String::from("");
    s.with_user_data(|data: &mut Data| {
        address = data.account.address.clone();
        let (_, balance) = lib::get_frontier_and_balance(address.clone(), &data.node_url);
        data.account.balance = balance;
        if data.account.balance > 0 {
            empty = false;
        }
    });

    if empty {
        let no_balance_message = format!("To send a message with dagchat you need a balance of at least 1 raw - a tiny fraction of a coin. One faucet claim will last you a lifetime. Your address is: {}", address);
        s.add_layer(
            Dialog::around(TextView::new(no_balance_message))
                .h_align(HAlign::Center)
                .button("Menu", |s| show_menu(s))
                .button("Copy Address", move |s| {
                    let mut clipboard = Clipboard::new().unwrap();
                    clipboard.set_text(address.clone()).unwrap();
                })
                .max_width(75),
        );
        return;
    }
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Recipient Address"))
                .child(TextArea::new().with_name("address"))
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
                            // Bring up the address book and allow choose recipient by name
                        })),
                )
                .child(DummyView)
                .child(TextView::new("Message Content"))
                .child(TextArea::new().with_name("message"))
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Send", |s| {
                            let mut address = String::from("");
                            let mut message = String::from("");
                            s.call_on_name("address", |view: &mut TextArea| {
                                address = String::from(view.get_content());
                            })
                            .unwrap();
                            s.call_on_name("message", |view: &mut TextArea| {
                                message = String::from(view.get_content());
                            })
                            .unwrap();
                            if address.is_empty() {
                                s.add_layer(Dialog::info(
                                    "You must provide an address to send the message to!",
                                ));
                                return;
                            }
                            if message.is_empty() {
                                s.add_layer(Dialog::info(
                                    "You must provide message content to send a message!",
                                ));
                                return;
                            }
                            let valid = lib::validate_address(&address);
                            if !valid {
                                s.add_layer(Dialog::info("The recipient's address is invalid."));
                                return;
                            }
                            process_send(s, message, address);
                        }))
                        .child(Button::new("Cancel", |s| show_menu(s))),
                ),
        )
        .title("Send a message")
        .padding_lrtb(1, 1, 1, 0),
    );
}

fn go_back(s: &mut Cursive) {
    s.pop_layer();
}

fn process_send(s: &mut Cursive, message: String, address: String) {
    let ticks = 1000;
    let cb = s.cb_sink().clone();
    let data = &s.user_data::<Data>().unwrap();
    let node_url = data.node_url.clone();
    let private_key_bytes = data.account.private_key;
    let prefix = data.prefix.clone();
    s.pop_layer();
    s.add_layer(Dialog::around(
        ProgressBar::new()
            .range(0, ticks)
            .with_task(move |counter| {
                lib::send_message(&private_key_bytes, address, message, &node_url, &prefix, &counter);
                cb.send(Box::new(show_sent)).unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}

fn show_sent(s: &mut Cursive) {
    s.set_autorefresh(false);
    s.pop_layer();
    show_menu(s);
    s.add_layer(Dialog::info("Message sent successfully!"));
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
                let mut receivables = lib::find_incoming(&target_address, &node_url);
                counter.tick(200);
                if !receivables.is_empty() {
                    let x = 800usize / receivables.len();
                    for receivable in &mut receivables {
                        if initial {
                            if receivable.amount == 1 {
                                receivable.message = lib::has_message(&receivable.hash, &node_url);
                            }
                        } else {
                            receivable.message = lib::has_message(&receivable.hash, &node_url);
                        }
                        counter.tick(x);
                    }
                }
                cb.send(Box::new(move |s| {
                    let data = &mut s.user_data::<Data>().unwrap();
                    data.account.receivables = receivables;
                    show_receive(s, initial);
                }))
                .unwrap();
            })
            .full_width(),
    ));
    s.set_autorefresh(true);
}

fn show_receive(s: &mut Cursive, initial: bool) {
    s.set_autorefresh(false);
    s.pop_layer();
    let data: Data = s.take_user_data().unwrap();

    if data.account.receivables.is_empty() {
        s.set_user_data(data);
        let content = "You don't have any receivables or incoming messages!";
        s.add_layer(Dialog::around(TextView::new(content)).button("Menu", |s| show_menu(s)));
        return;
    }

    let buttons = LinearLayout::vertical()
        .child(Button::new("Find messages", |s| {
            load_receivables(s, false)
        }))
        .child(Button::new("Refresh", |s| load_receivables(s, true)))
        .child(DummyView)
        .child(Button::new("Menu", |s| show_menu(s)));

    let select = SelectView::<String>::new()
        .on_submit(show_message_info)
        .with_name("select")
        .scrollable()
        .max_height(15);
    s.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(select)
                .child(DummyView)
                .child(buttons),
        )
        .title("Receivables"),
    );

    if initial {

        let mut info = StyledString::plain("Information for pre-alpha testers:\n");
        info.append(StyledString::styled("This is the inbox. Messages sent with 1 raw have already been identified as messages, but messages sent with an arbitrary amount will not yet have been detected. Select 'Find messages' from the buttons on the right to find these.", OFF_WHITE));
        s.add_layer(Dialog::around(TextView::new(info)).dismiss_button("Go to inbox").max_width(60));
    }
    for receivable in &data.account.receivables {
        let mut tag;
        if receivable.amount == 1 && receivable.message.is_some() {
            tag = String::from("Message");
        } else {
            if receivable.amount < 1000000 {
                tag = format!("{} raw", receivable.amount);
            } else {
                let raw = BigDecimal::from_str(&receivable.amount.to_string()).unwrap();
                let multi = BigDecimal::from_str("100000000000000000000000000000").unwrap();
                let x = raw / multi;
                tag = format!("{} {}", x.to_string(), data.ticker);
            }
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
    //eprintln!("Showmessageinfo");
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No receivable selected.")),
        Some(focus) => {
            let data = &mut s.user_data::<Data>().unwrap();
            //select.remove_item(focus);
            let receivable = &mut data.account.receivables[focus];
            //eprintln!("{:?}", receivable);
            let private_key = &data.account.private_key;
            let node_url = &data.node_url;
            let plaintext: String;
            if receivable.message.is_some() {
                let mut message = receivable.message.as_mut().unwrap();
                if message.plaintext.is_empty() {
                    let target = &message.head.as_mut().unwrap().contents.account;
                    let root_hash = &message.root_hash;
                    let blocks = message.blocks;
                    plaintext = lib::read_message(private_key, target, root_hash, blocks, node_url);
                    message.plaintext = plaintext.clone();
                } else {
                    plaintext = message.plaintext.clone();
                }
                let sender = receivable.source.clone();

                let buttons = LinearLayout::horizontal()
                .child(Button::new("Mark read", |s| {
                    //receive
                }))
                .child(Button::new("Back", |s| { go_back(s) }));

                s.add_layer(Dialog::around(LinearLayout::vertical()
                .child(TextView::new(plaintext).scrollable().max_size((65, 10)))
                .child(DummyView)
                .child(TextView::new("From"))
                .child(TextView::new(StyledString::styled(sender, OFF_WHITE)))
                .child(DummyView)
                .child(buttons)
                ).h_align(HAlign::Center).title("Message"));
            } else {
                s.add_layer(Dialog::info("no msg"));
            }
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

fn set_mnemonic(s: &mut Cursive, mnemonic: &str) {
    let entropy = lib::validate_mnemonic(mnemonic);
    let content;
    s.pop_layer();
    if !mnemonic.is_empty() && entropy.is_some() {
        let entropy_bytes = entropy.unwrap();
        let private_key_bytes = lib::get_private_key(&entropy_bytes);
        let private_key = ed25519_dalek::SecretKey::from_bytes(&private_key_bytes).unwrap();
        let public_key = ed25519_dalek::PublicKey::from(&private_key);
        let public_key_bytes = public_key.to_bytes();
        let prefix = &s.user_data::<Data>().unwrap().prefix;
        let address = lib::get_address(&public_key_bytes, prefix);
        s.with_user_data(|data: &mut Data| {
            data.account.entropy = entropy_bytes;
            data.account.private_key = private_key_bytes;
            data.account.public_key = public_key_bytes;
            data.account.address = address;
        });
        content = "Successfully imported account.";
        s.add_layer(Dialog::around(TextView::new(content)).button("Menu", |s| show_menu(s)));
    } else {
        content = "The mnemonic you entered was not valid.";
        s.add_layer(Dialog::around(TextView::new(content)).button("Back", |s| get_mnemonic(s)));
    }
}

fn show_menu(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::around(TextView::new("What would you like to do?"))
            .title("Main menu")
            .h_align(HAlign::Center)
            .button("Send", |s| show_send(s))
            .button("Receive", |s| load_receivables(s, true)),
    );
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
