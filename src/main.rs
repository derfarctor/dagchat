use bincode;
use clipboard::{ClipboardContext, ClipboardProvider};
use cursive::align::HAlign;
use cursive::theme::{BaseColor, BorderStyle, Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, OnEventView, ProgressBar, RadioGroup,
    SelectView, TextArea, TextView,
};
use cursive::Cursive;
use dirs;

use crate::util::constants::SHOW_TO_DP;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// dagchat util
mod dcutil;
use dcutil::*;

// dagchat wallets and messages util
mod messages;
mod wallets;
use messages::SavedMessage;

// send and receive with dagchat
mod receive;
mod send;

mod ui;
mod util;
use crate::ui::title::show_title;
use crate::util::constants::Colours::*;
use crate::util::constants::VERSION;
use crate::util::userdata::UserData;

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

fn show_change_rep(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let account = &wallet.accounts[wallet.acc_idx];
    let private_key = account.private_key;
    let coin = data.coin.clone();
    let address = account.address.clone();
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
                                let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                                let clip = clipboard
                                    .get_contents()
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
    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
    let data = &s.user_data::<UserData>().unwrap();
    let copied = clipboard.set_contents(string.clone());
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
    let wallet = &data.wallets[data.wallet_idx];
    let address = wallet.accounts[wallet.acc_idx].address.clone();
    let send_label = format!("Send {}", data.coin.name);
    let buttons = LinearLayout::vertical()
        .child(Button::new("Refresh", |s| receive::load_receivables(s)))
        .child(Button::new("Messages", |s| {
            let filter: messages::Filter = Default::default();
            messages::show_messages(s, filter);
        }))
        .child(DummyView)
        .child(Button::new(send_label, |s| send::show_send(s, false)))
        .child(Button::new("Send message", |s| send::show_send(s, true)))
        .child(DummyView)
        .child(Button::new("Copy address", move |s| {
            copy_to_clip(s, address.clone())
        }))
        .child(Button::new("Change rep", |s| show_change_rep(s)))
        .child(DummyView)
        .child(Button::new("Back", |s| wallets::show_accounts(s)));

    let select = SelectView::<String>::new()
        .on_submit(receive::show_message_info)
        .with_name("select")
        .scrollable()
        .fixed_height(5);

    let bal = display_to_dp(
        wallet.accounts[wallet.acc_idx].balance,
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

    for receivable in &wallet.accounts[wallet.acc_idx].receivables {
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
