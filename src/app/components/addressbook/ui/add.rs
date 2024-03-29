use super::primary::show_addressbook;
use crate::app::{
    clipboard::paste_clip, components::storage::save::save_to_storage, constants::colours::RED,
    helpers::go_back, themes::get_subtitle_colour, userdata::UserData,
};
use crate::crypto::address::validate_address;
use cursive::traits::{Nameable, Resizable};
use cursive::views::{Button, Dialog, DummyView, LinearLayout, TextArea, TextView, ViewRef};
use cursive::{utils::markup::StyledString, Cursive};

pub fn add_addressbook(s: &mut Cursive) {
    let data = &s.user_data::<UserData>().unwrap();
    let sub_title_colour = get_subtitle_colour(data.coins[data.coin_idx].colour);
    let form_content = LinearLayout::vertical()
        .child(TextView::new(StyledString::styled(
            "Name",
            sub_title_colour,
        )))
        .child(TextArea::new().with_name("contactname").max_width(66))
        .child(LinearLayout::horizontal().child(Button::new("Paste", |s| {
            let mut contactname: ViewRef<TextArea> = s.find_name("contactname").unwrap();
            contactname.set_content(paste_clip(s));
        })))
        .child(DummyView)
        .child(TextView::new(StyledString::styled(
            "Address",
            sub_title_colour,
        )))
        .child(TextArea::new().with_name("contactaddress").max_width(80))
        .child(LinearLayout::horizontal().child(Button::new("Paste", |s| {
            let mut contactaddress: ViewRef<TextArea> = s.find_name("contactaddress").unwrap();
            contactaddress.set_content(paste_clip(s));
        })))
        .child(DummyView)
        .child(DummyView)
        .child(
            LinearLayout::horizontal()
                .child(Button::new("Confirm", move |s| {
                    let mut name = String::from("");
                    let mut address = String::from("");
                    s.call_on_name("contactname", |view: &mut TextArea| {
                        name = String::from(view.get_content());
                    })
                    .unwrap();
                    s.call_on_name("contactaddress", |view: &mut TextArea| {
                        address = String::from(view.get_content());
                    })
                    .unwrap();

                    if !validate_address(&address) {
                        s.add_layer(Dialog::info("The contact's address is invalid."));
                        return;
                    }
                    let data = &mut s.user_data::<UserData>().unwrap();
                    let mut address_parts: Vec<&str> = address.split('_').collect();
                    let address = address_parts.pop().unwrap();
                    let mut old_address = String::from("");
                    for (contact_address, contact_name) in &data.addressbook {
                        if name == *contact_name {
                            old_address = contact_address.clone();
                            break;
                        }
                    }
                    if !old_address.is_empty() {
                        data.addressbook.remove(&old_address);
                    }
                    data.addressbook.insert(String::from("_") + address, name);
                    let saved = save_to_storage(s);
                    if let Ok(_saved) = saved {
                        s.pop_layer();
                        s.pop_layer();
                        show_addressbook(s);
                    } else {
                        s.add_layer(
                            Dialog::info(StyledString::styled(saved.err().unwrap(), RED))
                                .title("Error saving address book data."),
                        );
                    }
                }))
                .child(Button::new("Back", go_back)),
        );
    s.add_layer(
        Dialog::around(form_content)
            .title("Add Contact")
            .padding_lrtb(1, 1, 1, 0),
    );
}
