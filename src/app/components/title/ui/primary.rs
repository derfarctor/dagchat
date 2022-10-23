use super::super::checksetup::check_setup;
use crate::app::{coin::Coin, constants::VERSION, themes::set_theme, userdata::UserData};
use cursive::align::HAlign;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, RadioGroup};
use cursive::Cursive;

pub fn show_title(s: &mut Cursive) {
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
