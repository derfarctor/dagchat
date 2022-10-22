use super::constants::colours::OFF_WHITE;
use super::userdata::UserData;
use clipboard::{ClipboardContext, ClipboardProvider};
use cursive::theme::{BaseColor, Color};
use cursive::traits::Resizable;
use cursive::utils::markup::StyledString;
use cursive::views::{Dialog, TextView};
use cursive::Cursive;

pub fn copy_to_clip(s: &mut Cursive, string: String) {
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

pub fn paste_clip() -> String {
    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
    let clip = clipboard
        .get_contents()
        .unwrap_or_else(|_| String::from("Failed to read clipboard."));
    clip
}
