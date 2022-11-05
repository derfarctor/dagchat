use super::super::structs::WorkType;
use super::defaultrep::{get_default_rep_info, set_default_rep};
use super::nodeurl::{get_nodeurl_info, set_node_url};
use super::savemessages::{get_save_message_info, set_save_messages};
use super::workserverurl::set_work_server_url;
use super::worktype::{get_local_work_info, set_work_type};
use crate::app::clipboard::paste_clip;
use crate::app::components::storage::ui::setup::setup_password;
use crate::app::constants::colours::RED;
use crate::app::helpers::go_back;
use crate::app::themes::get_subtitle_colour;
use crate::app::userdata::UserData;
use crate::rpc::workgenerate::test_work_server;
use cursive::utils::markup::StyledString;
use cursive::view::Nameable;
use cursive::views::{
    Button, Dialog, DummyView, HideableView, LinearLayout, RadioGroup, ScreensView, TextArea,
    TextView, ViewRef,
};
use cursive::Cursive;

pub fn show_settings(s: &mut Cursive) {
    let data = &s.user_data::<UserData>().unwrap();
    let coin = &data.coins[data.coin_idx];
    let network = &coin.network;
    let default_rep = &network.default_rep;
    let coin_name = coin.name.clone();
    let node_url = network.node_url.clone();
    let work_server_url = network.work_server_url.clone();

    let mut work_type: RadioGroup<usize> = RadioGroup::new();
    let mut cpu_button = work_type.button(WorkType::CPU, "CPU");
    let mut boom_pow_button = work_type.button(WorkType::BOOMPOW, "BoomPow");
    let mut work_server_button = work_type.button(WorkType::WORK_SERVER, "Work Server");
    work_type.set_on_change(set_work_type);

    let mut save_messages: RadioGroup<bool> = RadioGroup::new();
    let mut save_encrypt_button = save_messages.button(true, "Save & Encrypt");
    let mut forget_button = save_messages.button(false, "Forget");
    save_messages.set_on_change(set_save_messages);
    if network.save_messages {
        save_encrypt_button.select();
    } else {
        forget_button.select();
    }

    let colour = get_subtitle_colour(coin.colour);
    let mut work_server_form = HideableView::new(
        LinearLayout::vertical()
            .child(DummyView)
            .child(TextView::new(StyledString::styled(
                "Work Server URL",
                colour,
            )))
            .child(
                TextArea::new()
                    .content(work_server_url)
                    .with_name("workserverurl"),
            )
            .child(
                LinearLayout::horizontal()
                    .child(Button::new("Change", |s| {
                        let mut work_server_url = String::from("");
                        s.call_on_name("workserverurl", |view: &mut TextArea| {
                            work_server_url = view.get_content().to_string();
                        })
                        .unwrap();
                        set_work_server_url(s, &work_server_url);
                    }))
                    .child(DummyView)
                    .child(Button::new("Paste", |s| {
                        let mut work_server_input: ViewRef<TextArea> =
                            s.find_name("workserverurl").unwrap();
                        work_server_input.set_content(paste_clip(s));
                    }))
                    .child(DummyView)
                    .child(Button::new("Test", move |s| {
                        let mut work_server_url = String::from("");
                        s.call_on_name("workserverurl", |view: &mut TextArea| {
                            work_server_url = view.get_content().to_string();
                        })
                        .unwrap();
                        let test = test_work_server(&work_server_url);
                        if let Ok(_) = test {
                            s.add_layer(Dialog::info(StyledString::styled(
                                "Communicated successfully with work server.",
                                colour,
                            )));
                        } else {
                            s.add_layer(Dialog::info(StyledString::styled(
                                format!(
                                    "Error communicating with work server: {}",
                                    test.err().unwrap()
                                ),
                                RED,
                            )));
                        }
                    })),
            ),
    );

    if network.work_type == WorkType::CPU {
        work_server_form.set_visible(false);
        cpu_button.select();
    } else if network.work_type == WorkType::BOOMPOW {
        work_server_form.set_visible(false);
        boom_pow_button.select();
    } else if network.work_type == WorkType::WORK_SERVER {
        work_server_form.set_visible(true);
        work_server_button.select();
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
                                    .child(Button::new("Info", get_nodeurl_info))
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
                                    .child(cpu_button)
                                    .child(DummyView)
                                    .child(DummyView)
                                    .child(work_server_button),
                            )
                            .child(work_server_form.with_name("hideable"))
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", get_local_work_info)),
                            ),
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
                    Dialog::around(LinearLayout::vertical().child(DummyView).child(
                        LinearLayout::horizontal().child(Button::new("Change", |s| {
                            setup_password(s, |s: &mut Cursive| {
                                s.add_layer(Dialog::info("Updated password successfully."))
                            })
                        })),
                    ))
                    .title("Application Password"),
                )
                .child(DummyView)
                .child(
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(save_encrypt_button)
                                    .child(DummyView)
                                    .child(DummyView)
                                    .child(forget_button),
                            )
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", get_save_message_info)),
                            ),
                    )
                    .title("Messages"),
                ),
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
                            .child(TextArea::new().content(default_rep).with_name("defaultrep"))
                            .child(DummyView)
                            .child(
                                LinearLayout::horizontal()
                                    .child(Button::new("Info", get_default_rep_info))
                                    .child(DummyView)
                                    .child(Button::new("Change", |s| {
                                        let mut default_rep = String::from("");
                                        s.call_on_name("defaultrep", |view: &mut TextArea| {
                                            default_rep = view.get_content().to_string();
                                        })
                                        .unwrap();
                                        set_default_rep(s, &default_rep);
                                    }))
                                    .child(DummyView)
                                    .child(Button::new("Paste", |s| {
                                        let mut default_rep: ViewRef<TextArea> =
                                            s.find_name("defaultrep").unwrap();
                                        default_rep.set_content(paste_clip(s));
                                    })),
                            ),
                    )
                    .title("Default Representative"),
                )
                .child(DummyView),
        )
        .title("Settings Page 3"),
    );

    s.add_layer(screens.with_name("settings"));
}
