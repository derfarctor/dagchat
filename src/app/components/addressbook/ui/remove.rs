use super::primary::show_addressbook;
use crate::app::{
    components::storage::save::save_to_storage, constants::colours::RED, userdata::UserData,
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
    let data = &mut s.user_data::<UserData>().unwrap();
    let address = data.addressbook.remove(name).unwrap();
    let saved = save_to_storage(s);
    if let Ok(_saved) = saved {
        s.pop_layer();
        show_addressbook(s);
    } else {
        let data = &mut s.user_data::<UserData>().unwrap();
        data.addressbook.insert(String::from(name), address);
        s.add_layer(
            Dialog::info(StyledString::styled(saved.err().unwrap(), RED))
                .title("Error saving address book data."),
        );
    }
}
