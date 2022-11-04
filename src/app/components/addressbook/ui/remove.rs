use super::primary::show_addressbook;
use crate::app::{
    components::storage::save::save_to_storage,
    constants::{colours::RED, AUTHOR},
    userdata::UserData,
};
use cursive::utils::markup::StyledString;
use cursive::views::{Dialog, OnEventView, SelectView};
use cursive::Cursive;

pub fn remove_addressbook(s: &mut Cursive) {
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

    if name == AUTHOR {
        return;
    }

    let data = &mut s.user_data::<UserData>().unwrap();
    let mut address = String::from("_ErrorAddressNotFound");

    // Inefficient, could use a reverse HashMap to go from value to key
    for (contact_address, contact_name) in &data.addressbook {
        if name == contact_name {
            address = contact_address.to_owned();
            break;
        }
    }
    data.addressbook.remove(&address).unwrap();

    let saved = save_to_storage(s);
    if let Ok(_saved) = saved {
        s.pop_layer();
        show_addressbook(s);
    } else {
        let data = &mut s.user_data::<UserData>().unwrap();
        data.addressbook.insert(address, String::from(name));
        s.add_layer(
            Dialog::info(StyledString::styled(saved.err().unwrap(), RED))
                .title("Error saving address book data."),
        );
    }
}
