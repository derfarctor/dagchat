use cursive::Cursive;
use crate::app::userdata::UserData;

pub fn show_addressbook<F>(s: &mut Cursive, on_success: F)
where
    F: Fn(&mut Cursive, String),
{
    let data = &mut s.user_data::<UserData>().unwrap();
    on_success(s, String::from("test"));
}