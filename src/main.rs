use arboard::Clipboard;
use cursive::align::HAlign;
use cursive::theme;
use cursive::traits::*;
use cursive::views::{DummyView, Button, Dialog, EditView, LinearLayout, TextArea, TextView};
use cursive::Cursive;

mod lib;

pub struct Data {
    pub account: Account,
    pub prefix: String,
    pub node_url: String
}

pub struct Account {
    pub entropy: [u8; 32],
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub address: String,
    pub balance: u128
}

impl Default for Account {
    fn default() -> Account {
        Account {
            entropy: [0u8; 32],
            private_key: [0u8; 32],
            public_key: [0u8; 32],
            address: String::from(""),
            balance: 0
        }
    }
}

impl Data {
    pub fn new() -> Self {
        Data {
            account: Default::default(),
            prefix: String::from("nano_"),
            node_url: String::from("https://app.natrium.io/api")
        }
    }
}

fn main() {
     /*
    let mnemonic = String::from("nice lock danger caught resemble limit rookie time laptop novel note oxygen mule tongue spray absent keep crowd mushroom mystery diesel dragon melody bone");
    let entropy = lib::validate_mnemonic(&mnemonic).unwrap();
    let p_key_bytes = lib::get_private_key(&entropy);
    let private_key = ed25519_dalek::SecretKey::from_bytes(&p_key_bytes).unwrap();
    let public_key = ed25519_dalek::PublicKey::from(&private_key);
    let address = lib::get_address(public_key.as_bytes(), "ban_");
    println!("{}", address);
    let node = "https://kaliumapi.appditto.com/api";
    let receivable = lib::find_incoming(address, node);
    println!("{:?}", receivable);
    //let first = &receivable[0];
    let message = lib::has_message("5762ECBD8F0176F411EEBB53CFF047218319724A1D2F816AFC646D024093A9A7", node);
    if let Some(message) = message {
        println!("{:?}", message);
        let plaintext = lib::read_message(private_key.as_bytes(), message, node);
        println!("Plaintext: {}", plaintext);
    } else {
        println!("Not a message");
    }
       */
   
    let mut siv = cursive::default();
    let data = Data::new();
    siv.set_user_data( data );
    siv.add_layer(
        Dialog::text("")
            .title("dagchat v.1.0.0")
            .h_align(HAlign::Center)
            .button("nano", |s| {
                set_theme(s, "nano");
                show_start(s);
            })
            .button("banano", |s| {
                set_theme(s, "banano");
                s.with_user_data(|data: &mut Data| {
                    data.prefix = String::from("ban_");
                    data.node_url = String::from("https://kaliumapi.appditto.com/api");
                });
                show_start(s);
            }),
    );
    siv.run();
 
}

fn show_start(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::text("Choose a way to import your banano wallet")
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
        let no_balance_message = format!("To send a message with dagchat you need a balance of at least 1 raw - a tiny fraction of a coin. Claim from a faucet to begin sending messages (one claim will last you a lifetime!). Your address is: {}", address);
        s.add_layer(Dialog::around(TextView::new(no_balance_message))
        .h_align(HAlign::Center)
        .button("Menu", |s| return_to_menu(s))
        .button("Copy Address", move |s| {
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(address.clone()).unwrap();
        })
        .max_width(75));
        return;
    }
    
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Recipient Address"))
                .child(TextArea::new().with_name("address"))
                .child(LinearLayout::horizontal()
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
                })))
                .child(DummyView)
                .child(TextView::new("Message Content"))
                .child(TextArea::new().with_name("message"))
                .child(DummyView)
                .child(LinearLayout::horizontal()
                .child(Button::new("Send", |s| {
                    let mut address = String::from("");
                    let mut message = String::from("");
                    s.call_on_name("address", |view: &mut TextArea| {
                        address = String::from(view.get_content());
                    }).unwrap();
                    s.call_on_name("message", |view: &mut TextArea| {
                        message = String::from(view.get_content());
                    }).unwrap();
                    if address.is_empty() {
                        s.add_layer(Dialog::info("You must provide an address to send the message to!"))
                    }
                    let valid = lib::validate_address(&address);
                    if !valid { s.add_layer(Dialog::info("The recipient's address is invalid.")) }
                    s.add_layer(Dialog::around(TextView::new("Sending message...")));

                    let data = &s.user_data::<Data>().unwrap();
                    let private_key_bytes = data.account.private_key;
                    let prefix = &data.prefix;
                    let node_url = &data.node_url;
                    lib::send_message(&private_key_bytes, address, message, node_url, prefix);
                    
                    // Message sent.
                    s.pop_layer();
                    s.add_layer(Dialog::info("Message sent successfully!"));
                }))
                .child(Button::new("Cancel", |s| return_to_menu(s)
                )))
        )
        .title("Send a message")
        .padding_lrtb(1, 1, 1, 0),
    );
}

fn go_back(s: &mut Cursive) {
    s.pop_layer();
}
fn show_receive(s: &mut Cursive) {
    s.pop_layer();
    //let mut select = SelectView::new()
        // Center the text horizontally
        //.h_align(HAlign::Center);
        
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
                    let mut clip = clipboard
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
    if  !mnemonic.is_empty() && entropy.is_some() {
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
        s.add_layer(Dialog::around(TextView::new(content)).button("Menu", |s| return_to_menu(s)));
    } else {
        content = "The mnemonic you entered was not valid.";
        s.add_layer(Dialog::around(TextView::new(content)).button("Back", |s| get_mnemonic(s)));
    }
}

fn return_to_menu(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::around(TextView::new("What would you like to do?"))
            .title("Main menu")
            .h_align(HAlign::Center)
            .button("Send", |s| show_send(s))
            .button("Receive", |s| show_receive(s))
            .button("Quit", |s| s.quit()),
    );
}

fn set_theme(s: &mut Cursive, style: &str) {
    let mut theme = s.current_theme().clone();
    if style == "nano" {
        theme = get_nano_theme(theme);
    } else {
        theme = get_banano_theme(theme);
    }
    s.set_theme(theme);
}

fn get_banano_theme(mut base: theme::Theme) -> theme::Theme {
    // USE RGB OR HEX
    base.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Black);
    base.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::Secondary] = theme::Color::Light(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::Tertiary] = theme::Color::Light(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::TitleSecondary] =
        theme::Color::Light(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::HighlightInactive] =
        theme::Color::Dark(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::Background] = theme::Color::Light(theme::BaseColor::Yellow);
    base.palette[theme::PaletteColor::Shadow] = theme::Color::Dark(theme::BaseColor::Yellow);
    base
}

fn get_nano_theme(mut base: theme::Theme) -> theme::Theme {
    // USE RGB OR HEx
    base.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::White);
    base.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::Blue);
    base.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Blue);
    base.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Blue);
    base.palette[theme::PaletteColor::HighlightInactive] =
        theme::Color::Dark(theme::BaseColor::Blue);
    base.palette[theme::PaletteColor::Background] = theme::Color::Light(theme::BaseColor::Blue);
    base.palette[theme::PaletteColor::Shadow] = theme::Color::Dark(theme::BaseColor::Blue);
    base
}
