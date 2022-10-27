use crate::app::userdata::UserData;
use cursive::{views::TextArea, Cursive};

pub fn show_addressbook(s: &mut Cursive) {
    let data = &mut s.user_data::<UserData>().unwrap();
    s.call_on_name("address", |view: &mut TextArea| {
        view.set_content("This is a test");
    })
    .unwrap();
}
