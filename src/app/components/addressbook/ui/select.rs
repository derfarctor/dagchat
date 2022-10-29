use crate::app::userdata::UserData;
use cursive::{
    views::{Dialog, HideableView, TextArea},
    Cursive,
};

pub fn select_addressbook(s: &mut Cursive, name: &str) {
    let data = &s.user_data::<UserData>().unwrap();
    let address = &data.addressbook.get(name).unwrap().clone();
    s.call_on_name("hideable", |view: &mut HideableView<Dialog>| {
        view.set_visible(true);
    })
    .unwrap();
    s.call_on_name("address", |view: &mut TextArea| {
        view.set_content(address);
    })
    .unwrap();
    s.pop_layer();
}
