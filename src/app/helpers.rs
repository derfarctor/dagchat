use cursive::views::EditView;
use cursive::Cursive;

pub fn go_back(s: &mut Cursive) {
    s.pop_layer();
}

pub fn get_name(s: &mut Cursive) -> String {
    let name = s
        .call_on_name("name", |view: &mut EditView| view.get_content())
        .unwrap();
    s.pop_layer();
    name.to_string()
}
