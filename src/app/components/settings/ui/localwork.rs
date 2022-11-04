use crate::app::{
    components::storage::save::save_to_storage, constants::colours::RED,
    themes::get_subtitle_colour, userdata::UserData,
};
use cursive::{utils::markup::StyledString, views::Dialog, Cursive};

pub fn set_local_work(s: &mut Cursive, local_work: &usize) {
    let data = &mut s.user_data::<UserData>().unwrap();
    data.coins[data.coin_idx].network.work_type = *local_work;
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
            StyledString::styled("\nEach transaction you make will require a small proof of work to be attached when it is published to the network.\n\nAs such you can choose whether or not to outsource this calculation by selecting the BoomPow option, which will reduce the workload required by your computer (This is especially apparent when sending messages on nano where you may be waiting a long time to generate work locally).\n\nIn the event that the BoomPow API is no longer working and you are having repeated errors making transactions, setting this option to local work will guarantee successful work generation.", colour),
        ).title("Proof of Work"));
}
