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
}
