use crate::app::{
    components::storage::save::save_to_storage, constants::colours::RED,
    themes::get_subtitle_colour, userdata::UserData,
};
use cursive::{utils::markup::StyledString, views::Dialog, Cursive};

pub fn set_work_server_url(s: &mut Cursive, work_server_url: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    data.coins[data.coin_idx].network.work_server_url = String::from(work_server_url);
    let colour = get_subtitle_colour(data.coins[data.coin_idx].colour);
    let saved = save_to_storage(s);
    if let Ok(_saved) = saved {
        s.add_layer(Dialog::info(StyledString::styled(
            "Updated work server URL successfully.",
            colour,
        )));
    } else {
        s.add_layer(Dialog::info(StyledString::styled(
            format!("Failed to save work server URL. {}", saved.err().unwrap()),
            RED,
        )));
    }
}
