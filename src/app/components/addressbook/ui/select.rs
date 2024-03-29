use crate::app::userdata::UserData;
use cursive::{
    views::{Dialog, HideableView, TextArea},
    Cursive,
};

use super::info::contact_info;

pub fn select_addressbook(s: &mut Cursive, name: &str) {
    if s.call_on_name("address", |_: &mut TextArea| {}).is_none() {
        contact_info(s);
        return;
    }
    s.pop_layer();

    let data = &s.user_data::<UserData>().unwrap();

    let mut address = String::from("_ErrorAddressNotFound");

    // Inefficient, could use a reverse HashMap to go from value to key
    for (contact_address, contact_name) in &data.addressbook {
        if name == contact_name {
            address = contact_address.to_owned();
            break;
        }
    }
    let idx_of_ = address.find('_').unwrap();
    let prefix = &address[..idx_of_ + 1];
    let network_prefix = &data.coins[data.coin_idx].prefix;
    if network_prefix != prefix {
        let non_prefix = &address[idx_of_ + 1..];
        address = network_prefix.to_owned() + non_prefix
    }

    s.call_on_name("hideable", |view: &mut HideableView<Dialog>| {
        view.set_visible(true);
    })
    .unwrap();
    s.call_on_name("address", |view: &mut TextArea| {
        view.set_content(address);
    })
    .unwrap();
}
