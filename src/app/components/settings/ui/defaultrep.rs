use crate::{
    app::{
        components::storage::save::save_to_storage, constants::colours::RED,
        themes::get_subtitle_colour, userdata::UserData,
    },
    crypto::address::validate_address,
};
use cursive::{utils::markup::StyledString, views::Dialog, Cursive};

pub fn set_default_rep(s: &mut Cursive, default_rep: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let colour = get_subtitle_colour(data.coins[data.coin_idx].colour);
    if validate_address(default_rep) {
        data.coins[data.coin_idx].network.default_rep = String::from(default_rep);
        let saved = save_to_storage(s);
        if let Ok(_saved) = saved {
            s.add_layer(Dialog::info(StyledString::styled(
                "Updated default representative successfully.",
                colour,
            )));
        } else {
            s.add_layer(Dialog::info(StyledString::styled(
                format!(
                    "Failed to save default representative. {}",
                    saved.err().unwrap()
                ),
                RED,
            )));
        }
    } else {
        s.add_layer(Dialog::info(StyledString::styled(
            "The default representative address was invalid.",
            RED,
        )));
    }
}

pub fn get_default_rep_info(s: &mut Cursive) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let colour = data.coins[data.coin_idx].colour;
    s.add_layer(Dialog::info(StyledString::styled("\nThe default representative is the representative account that any accounts, which are newly opened within the dagchat wallet, will use as their representative.", colour)
        ).title("Default Representative"));
}
