use crate::app::{
    components::storage::save::save_to_storage, constants::colours::RED,
    themes::get_subtitle_colour, userdata::UserData,
};
use cursive::{utils::markup::StyledString, views::Dialog, Cursive};

pub fn set_save_messages(s: &mut Cursive, save_messages: &bool) {
    let data = &mut s.user_data::<UserData>().unwrap();
    data.coins[data.coin_idx].network.save_messages = *save_messages;
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

pub fn get_save_message_info(s: &mut Cursive) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let colour = data.coins[data.coin_idx].colour;
    s.add_layer(Dialog::info(
            StyledString::styled("\nYou can choose whether you want dagchat to Save & Encrypt the messages you have sent and received, for reference in the future. Saved messages can be read, filtered and searched via the <Messages log> button which is accessible from the inbox once you have loaded an account.", colour),
        ).title("Messages"));
}
