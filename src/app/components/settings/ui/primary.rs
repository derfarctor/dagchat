use crate::app::clipboard::paste_clip;
use crate::app::components::storage::save::save_to_storage;
use crate::app::helpers::go_back;
use crate::app::userdata::UserData;
use cursive::view::Nameable;
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, RadioGroup, ScreensView, TextArea, ViewRef,
};
use cursive::Cursive;

pub fn show_settings(s: &mut Cursive) {
    let data = &s.user_data::<UserData>().unwrap();
    let coin = &data.coins[data.coin_idx];
    let network = &coin.network;
    let coin_name = coin.name.clone();
    let node_url = network.node_url.clone();

    let mut local_work: RadioGroup<bool> = RadioGroup::new();
    let mut local_work_button = local_work.button(true, "Local");
    let mut boom_pow_button = local_work.button(false, "Boom Pow");
    local_work.set_on_change(set_local_work);
    if network.local_work {
        local_work_button.select();
    } else {
        boom_pow_button.select();
    }
    let mut screens = ScreensView::new();

    screens.add_active_screen(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Back", go_back))
                        .child(DummyView)
                        .child(Button::new("Next page", move |s| {
                            s.call_on_name("settings", |view: &mut ScreensView<Dialog>| {
                                view.set_active_screen(view.active_screen() + 1);
                            })
                            .unwrap();
                        })),
                )
                .child(DummyView)
                .child(
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(TextArea::new().content(node_url).with_name("nodeurl"))
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", |s| {}))
                                    .child(DummyView)
                                    .child(Button::new("Change", |s| {
                                        let mut node_url = String::from("");
                                        s.call_on_name("nodeurl", |view: &mut TextArea| {
                                            node_url = view.get_content().to_string();
                                        })
                                        .unwrap();
                                        set_node_url(s, &node_url);
                                    }))
                                    .child(DummyView)
                                    .child(Button::new("Paste", |s| {
                                        let mut node_url: ViewRef<TextArea> =
                                            s.find_name("nodeurl").unwrap();
                                        node_url.set_content(paste_clip(s));
                                    })),
                            ),
                    )
                    .title(format!(
                        "{} node API",
                        coin_name[0..1].to_uppercase() + &coin_name[1..]
                    )),
                )
                .child(DummyView)
                .child(
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(boom_pow_button)
                                    .child(DummyView)
                                    .child(DummyView)
                                    .child(local_work_button),
                            )
                            .child(DummyView)
                            .child(LinearLayout::horizontal().child(Button::new("Info", |s| {}))),
                    )
                    .title("Proof of Work"),
                )
                .child(DummyView),
        )
        .title("Settings Page 1"),
    );

    screens.add_screen(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Back", go_back))
                        .child(DummyView)
                        .child(Button::new("Next page", move |s| {
                            s.call_on_name("settings", |view: &mut ScreensView<Dialog>| {
                                view.set_active_screen(view.active_screen() + 1);
                            })
                            .unwrap();
                        })),
                )
                .child(DummyView)
                .child(
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(TextArea::new().content("Setting 3"))
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", |s| {}))
                                    .child(DummyView)
                                    .child(Button::new("Change", |s| {})),
                            ),
                    )
                    .title("Setting 3"),
                )
                .child(DummyView)
                .child(
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(TextArea::new().content("Setting 4"))
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", |s| {}))
                                    .child(DummyView)
                                    .child(Button::new("Change", |s| {})),
                            ),
                    )
                    .title("Setting 4"),
                )
                .child(DummyView),
        )
        .title("Settings Page 2"),
    );

    screens.add_screen(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Back", go_back))
                        .child(DummyView)
                        .child(Button::new("Next page", move |s| {
                            s.call_on_name("settings", |view: &mut ScreensView<Dialog>| {
                                view.set_active_screen(view.active_screen() - 2);
                            })
                            .unwrap();
                        })),
                )
                .child(DummyView)
                .child(
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(TextArea::new().content("Setting 5"))
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", |s| {}))
                                    .child(DummyView)
                                    .child(Button::new("Change", |s| {})),
                            ),
                    )
                    .title("Setting 5"),
                )
                .child(DummyView)
                .child(
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(TextArea::new().content("Setting 6"))
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", |s| {}))
                                    .child(DummyView)
                                    .child(Button::new("Change", |s| {})),
                            ),
                    )
                    .title("Setting 6"),
                )
                .child(DummyView),
        )
        .title("Settings Page 3"),
    );

    s.add_layer(screens.with_name("settings"));
}

fn set_local_work(s: &mut Cursive, local_work: &bool) {
    let data = &mut s.user_data::<UserData>().unwrap();
    data.coins[data.coin_idx].network.local_work = *local_work;
    let saved = save_to_storage(s);
    if let Ok(_saved) = saved {
        s.add_layer(Dialog::info("Updated selection successfully."));
    } else {
        s.add_layer(Dialog::info("Error saving local work option."));
    }
}

fn set_node_url(s: &mut Cursive, node_url: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    data.coins[data.coin_idx].network.node_url = String::from(node_url);
    let saved = save_to_storage(s);
    if let Ok(_saved) = saved {
        s.add_layer(Dialog::info("Updated node API successfully."));
    } else {
        s.add_layer(Dialog::info("Error saving node API."));
    }
}
