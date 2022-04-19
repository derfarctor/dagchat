use arboard::Clipboard;
use bincode;
use cursive::align::HAlign;
use cursive::theme::{BaseColor, BorderStyle, Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ProgressBar, RadioGroup, SelectView,
    TextArea, TextView,
};
use cursive::Cursive;
use dirs;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub mod defaults;
use defaults::{ACCOUNTS_PATH, DATA_DIR_PATH, MESSAGES_DIR_PATH, SHOW_TO_DP};
// dagchat util
mod dcutil;
use dcutil::*;

// dagchat accounts and messages util
mod accounts;
mod messages;
use messages::SavedMessage;

// send and receive with dagchat
mod receive;
mod send;

pub struct UserData {
    pub password: String,
    pub accounts: Vec<accounts::Account>,
    pub acc_idx: usize,
    pub acc_messages: Result<Vec<SavedMessage>, String>,
    pub lookup: HashMap<String, String>,
    pub coin: Coin,
    pub encrypted_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Coin {
    prefix: String,
    name: String,
    ticker: String,
    multiplier: String,
    node_url: String,
    colour: Color,
}

impl Coin {
    fn nano() -> Coin {
        Coin {
            prefix: String::from("nano_"),
            name: String::from("nano"),
            ticker: String::from("Ӿ"),
            multiplier: String::from("1000000000000000000000000000000"),
            node_url: String::from("https://app.natrium.io/api"),
            colour: L_BLUE,
        }
    }
    fn banano() -> Coin {
        Coin {
            prefix: String::from("ban_"),
            name: String::from("banano"),
            ticker: String::from(" BAN"),
            multiplier: String::from("100000000000000000000000000000"),
            node_url: String::from("https://kaliumapi.appditto.com/api"),
            colour: YELLOW,
        }
    }
}

impl UserData {
    pub fn new() -> Self {
        UserData {
            password: String::from(""),
            accounts: vec![],
            acc_idx: 0,
            lookup: HashMap::new(),
            acc_messages: Ok(vec![]),
            coin: Coin::nano(),
            encrypted_bytes: vec![],
        }
    }
}

const VERSION: &str = "beta v1.0.0";

const L_BLUE: Color = Color::Rgb(62, 138, 227);
const M_BLUE: Color = Color::Rgb(0, 106, 255);
const D_BLUE: Color = Color::Rgb(12, 37, 125);

const YELLOW: Color = Color::Light(BaseColor::Yellow);
const OFF_WHITE: Color = Color::Rgb(245, 245, 247);
const RED: Color = Color::Light(BaseColor::Red);

fn main() {
    let backend_init = || -> std::io::Result<Box<dyn cursive::backend::Backend>> {
        let backend = cursive::backends::crossterm::Backend::init()?;
        let buffered_backend = cursive_buffered_backend::BufferedBackend::new(backend);
        Ok(Box::new(buffered_backend))
    };

    let mut siv = cursive::default();
    siv.set_window_title(format!("dagchat {}", VERSION));

    show_title(&mut siv);

    siv.try_run_with(backend_init).ok().unwrap();
}

fn show_title(s: &mut Cursive) {
    let data = UserData::new();
    s.set_user_data(data);

    set_theme(s, "nano", false);
    let mut theme_group: RadioGroup<bool> = RadioGroup::new();

    let mut coin_group: RadioGroup<String> = RadioGroup::new();

    let radios = LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
                .child(coin_group.button(String::from("nano"), "nano").selected())
                .child(coin_group.button(String::from("banano"), "banano")),
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
            s.with_user_data(|data: &mut UserData| {
                data.coin = Coin::banano();
            });
        }
        check_setup(s);
    });

    s.add_layer(
        Dialog::new()
            .content(
                LinearLayout::vertical()
                    .child(DummyView)
                    .child(button)
                    .child(DummyView)
                    .child(radios),
            )
            .title(format!("dagchat {}", VERSION))
            .h_align(HAlign::Center),
    );
}

fn go_back(s: &mut Cursive) {
    s.pop_layer();
}

fn get_subtitle_colour(s: &mut Cursive) -> Color {
    let data = &s.user_data::<UserData>().unwrap();
    let sub_title_colour;
    if data.coin.colour == YELLOW {
        sub_title_colour = OFF_WHITE;
    } else {
        sub_title_colour = data.coin.colour;
    }
    sub_title_colour
}

fn check_setup(s: &mut Cursive) {
    if let Some(data_dir) = dirs::data_dir() {
        let dagchat_dir = data_dir.join(DATA_DIR_PATH);
        let messages_dir = dagchat_dir.join(MESSAGES_DIR_PATH);
        if !dagchat_dir.exists() {
            fs::create_dir(&dagchat_dir).unwrap_or_else(|e| {
                let content = format!(
                    "Failed to create a data folder for dagchat at path: {:?}\nError: {}",
                    dagchat_dir, e
                );
                s.add_layer(Dialog::info(content))
            });
            if !dagchat_dir.exists() {
                return;
            }
        }
        if !messages_dir.exists() {
            fs::create_dir(&messages_dir).unwrap_or_else(|e| {
                let content = format!(
                    "Failed to create a messages folder for dagchat at path: {:?}\nError: {}",
                    messages_dir, e
                );
                s.add_layer(Dialog::info(content))
            });
            if !messages_dir.exists() {
                return;
            }
        }
        accounts::load_accounts(s, dagchat_dir);
    } else {
        s.add_layer(Dialog::info(
            "Error locating the application data folder on your system.",
        ));
    }
}

fn show_change_rep(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let private_key = data.accounts[data.acc_idx].private_key;
    let coin = data.coin.clone();
    let address = data.accounts[data.acc_idx].address.clone();
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
                            "You must provide an address to change representative to!"
                        ));
                        return;
                    }
                    let valid = validate_address(&rep_address);
                    if !valid {
                        s.add_layer(Dialog::info("The representative's address is invalid."));
                        return;
                    }
                    let account_info_opt = get_account_info(&address, &coin.node_url);
                    if account_info_opt.is_none() {
                        s.add_layer(Dialog::info(format!("You can't change representatives until you open your account by receiving some {}.", coin.name)));
                        return;
                    }
                    let account_info = account_info_opt.unwrap();
                    change_rep(&private_key, account_info, &rep_address, &coin.node_url, &coin.prefix);
                    s.pop_layer();
                    show_inbox(s);
                    s.add_layer(Dialog::info("Successfully changed representative!"));
                }))
                .child(Button::new("Back", |s| show_inbox(s)))),
        )
        .title("Change representative"),
    );
}

fn copy_to_clip(s: &mut Cursive, string: String) {
    let mut clipboard = Clipboard::new().unwrap();
    let data = &s.user_data::<UserData>().unwrap();
    let copied = clipboard.set_text(string.clone());
    if copied.is_err() {
        s.add_layer(Dialog::info(StyledString::styled(
            "Error copying to clipboard.",
            Color::Light(BaseColor::Red),
        )));
    } else {
        let mut content = StyledString::styled(format!("{}\n", string), OFF_WHITE);
        content.append(StyledString::styled(
            "was successfully copied to your clipboard.",
            data.coin.colour,
        ));
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
    let data: UserData = s.take_user_data().unwrap();
    let address = data.accounts[data.acc_idx].address.clone();
    let send_label = format!("Send {}", data.coin.name);
    let buttons = LinearLayout::vertical()
        .child(Button::new("Refresh", |s| receive::load_receivables(s)))
        .child(DummyView)
        .child(Button::new(send_label, |s| send::show_send(s, false)))
        .child(Button::new("Send message", |s| send::show_send(s, true)))
        .child(DummyView)
        .child(Button::new("Copy address", move |s| {
            copy_to_clip(s, address.clone())
        }))
        .child(Button::new("Change rep", |s| show_change_rep(s)))
        .child(DummyView)
        .child(Button::new("Back", |s| accounts::show_accounts(s)));

    let select = SelectView::<String>::new()
        .on_submit(receive::show_message_info)
        .with_name("select")
        .scrollable()
        .max_height(6);

    let bal = display_to_dp(
        data.accounts[data.acc_idx].balance,
        SHOW_TO_DP,
        &data.coin.multiplier,
        &data.coin.ticker,
    );
    let bal_text = format!("Balance: {}", bal);
    let bal_content =
        TextView::new(StyledString::styled(bal_text, data.coin.colour)).with_name("balance");
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

    for receivable in &data.accounts[data.acc_idx].receivables {
        let mut tag;
        if receivable.amount == 1 && receivable.message.is_some() {
            tag = String::from("Message");
        } else {
            tag = display_to_dp(
                receivable.amount,
                SHOW_TO_DP,
                &data.coin.multiplier,
                &data.coin.ticker,
            );
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
