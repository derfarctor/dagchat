use crate::app::{
    clipboard::copy_to_clip, constants::colours::OFF_WHITE, helpers::go_back, userdata::UserData,
};
use cursive::{
    utils::markup::StyledString,
    views::{Dialog, OnEventView, SelectView},
    Cursive,
};

pub fn contact_info(s: &mut Cursive) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("addressbook")
        .unwrap();
    let select = eventview.get_inner();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No contact selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let (name, _) = select.get_item(focus).unwrap();

    let data = &s.user_data::<UserData>().unwrap();
    let mut address = String::from("_ErrorAddressNotFound");

    // Inefficient, could use a reverse HashMap to go from value to key
    for (contact_address, contact_name) in &data.addressbook {
        if name == contact_name {
            address = contact_address.to_owned();
        }
    }
    let idx_of_ = address.find('_').unwrap();
    let prefix = &address[..idx_of_ + 1];
    let network_prefix = &data.coins[data.coin_idx].prefix;
    if network_prefix != prefix {
        let non_prefix = &address[idx_of_ + 1..];
        address = network_prefix.to_owned() + non_prefix
    }
    let colour = data.coins[data.coin_idx].colour;

    let mut contact_info = StyledString::styled("Name\n", colour);
    contact_info.append(StyledString::styled(name, OFF_WHITE));
    contact_info.append(StyledString::styled("\n\nAddress\n", colour));
    contact_info.append(StyledString::styled(&address, OFF_WHITE));
    s.add_layer(
        Dialog::text(contact_info)
            .button("Back", go_back)
            .button("Copy address", move |s| copy_to_clip(s, address.clone())),
    );
}
