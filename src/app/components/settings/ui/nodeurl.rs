use crate::app::{components::storage::save::save_to_storage, userdata::UserData};
use cursive::{utils::markup::StyledString, views::Dialog, Cursive};

pub fn set_node_url(s: &mut Cursive, node_url: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    data.coins[data.coin_idx].network.node_url = String::from(node_url);
    let saved = save_to_storage(s);
    if let Ok(_saved) = saved {
        s.add_layer(Dialog::info("Updated node API successfully."));
    } else {
        s.add_layer(Dialog::info(format!(
            "Failed to save node API. {}",
            saved.err().unwrap()
        )));
    }
}

pub fn get_nodeurl_info(s: &mut Cursive) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let colour = data.coins[data.coin_idx].colour;
    s.add_layer(Dialog::info(StyledString::styled("\nThis is the URL for the node's API that you wish to communicate with using the dagchat wallet.", colour)).title("Node API"));
}
