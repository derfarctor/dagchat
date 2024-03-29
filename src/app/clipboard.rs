use super::constants::colours::{OFF_WHITE, RED};
use super::userdata::UserData;
use cursive::traits::Resizable;
use cursive::utils::markup::StyledString;
use cursive::views::{Dialog, TextView};
use cursive::Cursive;

pub fn copy_to_clip(s: &mut Cursive, string: String) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let copied = data.clipboard.set_text(string.clone());
    if copied.is_err() {
        s.add_layer(Dialog::info(StyledString::styled(
            "Error copying to clipboard.",
            RED,
        )));
    } else {
        let mut content = StyledString::styled(format!("{}\n", string), OFF_WHITE);
        content.append(StyledString::styled(
            "was successfully copied to your clipboard.",
            data.coins[data.coin_idx].colour,
        ));
        s.add_layer(
            Dialog::around(TextView::new(content))
                .dismiss_button("Back")
                .max_width(80),
        );
    }
}

pub fn paste_clip(s: &mut Cursive) -> String {
    let data = &mut s.user_data::<UserData>().unwrap();
    data.clipboard
        .get_text()
        .unwrap_or_else(|_| String::from("Failed to read clipboard."))
}
