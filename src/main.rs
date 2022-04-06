use arboard::Clipboard;
use cursive::align::HAlign;
use cursive::theme;
use cursive::traits::*;
use cursive::views::{Dialog, EditView, TextView};
use cursive::Cursive;

mod lib;

fn main() {

    let mut siv = cursive::default();
    siv.add_layer(
        Dialog::text("")
            .title("dagchat v.1.0.0")
            .h_align(HAlign::Center)
            .button("nano", |s| {
                set_theme(s, "nano");
                load_start(s);
            })
            .button("banano", |s| {
                set_theme(s, "banano");
                load_start(s);
            }),
    );
    siv.run();
}

fn load_start(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::text("Please choose how you would like to load your account")
            .title("Import account")
            .h_align(HAlign::Center)
            .button("mnemonic", |s| {
                get_mnemonic(s, false);
            }),
    );
}

fn get_mnemonic(s: &mut Cursive, retrying: bool) {
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
            .button("Ok", |s| {
                let mnemonic = s
                    .call_on_name("mnemonic", |view: &mut EditView| {
                        view.get_content()
                    })
                    .unwrap();

                set_mnemonic(s, &mnemonic);
            })
            .button("Paste", |s| {
                s.call_on_name("mnemonic", |view: &mut EditView| {
                    let mut clipboard = Clipboard::new().unwrap();
                    let mut clip = clipboard.get_text().unwrap_or_else(|_| {
                        String::from("Failed to read clipboard.")
                    });
                    view.set_content(clip);
                })
                .unwrap();
            })
            .button("Back", |s| {
                load_start(s);
            }),
    );
}

fn set_mnemonic(s: &mut Cursive, mnemonic: &str) {
    let (entropy, valid) = lib::validate_mnemonic(mnemonic);
    let content;
    s.pop_layer();
    if valid && !mnemonic.is_empty() {
        content = "Successfully imported account.";
        s.add_layer(Dialog::around(TextView::new(content)).button("Menu", |s| return_to_menu(s)));
    } else {
        content = "The mnemonic you entered was not valid.";
        s.add_layer(
            Dialog::around(TextView::new(content))
                .button("Back", |s| get_mnemonic(s, true))
        );
    }
}

fn return_to_menu(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::around(TextView::new("What would you like to do?"))
            .title("Main menu")
            .h_align(HAlign::Center)
            .button("Send", |s| {

            })
            .button("Receive", |s| {

            })
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
    base.palette[theme::PaletteColor::TitleSecondary] = theme::Color::Light(theme::BaseColor::Yellow);
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
