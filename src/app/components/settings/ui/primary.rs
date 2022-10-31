use cursive::views::{
    Button, Dialog, DummyView, HideableView, LinearLayout, TextArea, TextView, ViewRef,
};
use cursive::Cursive;

use crate::app::helpers::go_back;
use crate::app::userdata::UserData;

pub fn show_settings(s: &mut Cursive) {
    let data = &s.user_data::<UserData>().unwrap();
    let coin = &data.coins[data.coin_idx];
    let network = &coin.network;
    let coin_name = coin.name.clone();
    let node_url = network.node_url.clone();
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(Button::new("Back", go_back))
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new(format!(
                            "{} node API",
                            coin_name[0..1].to_uppercase() + &coin_name[1..]
                        )))
                        .child(DummyView)
                        .child(Button::new("Change", |s| {}))
                        .child(DummyView)
                        .child(Button::new("Info", |s| {})),
                )
                .child(TextArea::new().content(node_url))
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new("Setting2"))
                        .child(DummyView)
                        .child(Button::new("Change", |s| {}))
                        .child(DummyView)
                        .child(Button::new("Info", |s| {})),
                )
                .child(TextArea::new())
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new("Setting3"))
                        .child(DummyView)
                        .child(Button::new("Change", |s| {}))
                        .child(DummyView)
                        .child(Button::new("Info", |s| {})),
                )
                .child(TextArea::new())
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(TextView::new("Setting4"))
                        .child(DummyView)
                        .child(Button::new("Change", |s| {}))
                        .child(DummyView)
                        .child(Button::new("Info", |s| {})),
                )
                .child(TextArea::new())
                .child(DummyView),
        )
        .title("Settings"),
    );
}
