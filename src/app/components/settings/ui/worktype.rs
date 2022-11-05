use crate::app::{
    components::{settings::structs::WorkType, storage::save::save_to_storage},
    constants::colours::RED,
    themes::get_subtitle_colour,
    userdata::UserData,
};
use cursive::{
    utils::markup::StyledString,
    views::{Dialog, HideableView, LinearLayout},
    Cursive,
};

pub fn set_work_type(s: &mut Cursive, work_type: &usize) {
    if *work_type != WorkType::WORK_SERVER {
        s.call_on_name("hideable", |view: &mut HideableView<LinearLayout>| {
            view.set_visible(false);
        })
        .unwrap();
    } else {
        s.call_on_name("hideable", |view: &mut HideableView<LinearLayout>| {
            view.set_visible(true);
        })
        .unwrap();
    }
    let data = &mut s.user_data::<UserData>().unwrap();
    data.coins[data.coin_idx].network.work_type = *work_type;
    let colour = get_subtitle_colour(data.coins[data.coin_idx].colour);
    let saved = save_to_storage(s);
    if let Ok(_saved) = saved {
        s.add_layer(Dialog::info(StyledString::styled(
            "Updated selection successfully.",
            colour,
        )));
    } else {
        s.add_layer(Dialog::info(StyledString::styled(
            format!("Failed to save selection. {}", saved.err().unwrap()),
            RED,
        )));
    }
}

pub fn get_local_work_info(s: &mut Cursive) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let colour = data.coins[data.coin_idx].colour;
    s.add_layer(Dialog::info(
            StyledString::styled("\nEach transaction you make will require a small proof of work to be attached when it is published to the network.\n\nAs such you can choose whether or not to outsource this calculation by selecting the BoomPow option, which will reduce the workload required by your computer (This is especially apparent when sending messages on nano where you may be waiting a long time to generate work locally).\n\nIn the event that the BoomPow API is not working (you are having errors sending and receiving with kalium/natrium in the error message), setting this option to CPU will guarantee successful work generation.\n\nYou can also choose to use a custom nano work server.", colour),
        ).title("Proof of Work"));
}
