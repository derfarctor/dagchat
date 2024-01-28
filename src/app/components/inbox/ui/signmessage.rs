use crate::app::constants::colours::RED;
use crate::crypto::signmessage::sign_message;
use crate::app::{
    clipboard::paste_clip,
    clipboard::copy_to_clip,
    helpers::go_back,
    themes::get_subtitle_colour,
    userdata::UserData
};
use super::primary::show_inbox;
use cursive::Cursive;
use cursive::views::{
  HideableView,
  Button,
  Dialog,
  LinearLayout,
  DummyView,
  TextView,
  TextArea,
  ViewRef,
};
use cursive::traits::Nameable;
use cursive::utils::markup::StyledString;

pub fn show_sign_message(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let account = &wallet.accounts[wallet.acc_idx];
    let private_key = account.private_key;
    let coin = data.coins[data.coin_idx].clone();
    let sub_title_colour = get_subtitle_colour(coin.colour);
    s.add_layer(
      HideableView::new(
      Dialog::around(
            LinearLayout::vertical()
            .child(DummyView)
                .child(TextView::new(StyledString::styled(
                    "Message signing",
                    sub_title_colour,
                )))
                .child(TextArea::new().with_name("message"))
                .child(
                    LinearLayout::horizontal()
                        .child(Button::new("Paste", |s| {
                            let mut message: ViewRef<TextArea> = s.find_name("message").unwrap();
                            message.set_content(paste_clip(s));
                        }))
                )
                //
                .child(DummyView)
                .child(LinearLayout::horizontal()
                .child(Button::new("Sign", move |s| {
                    let mut message = String::from("");
                    s.call_on_name("message", |view: &mut TextArea| {
                        message = String::from(view.get_content());
                    });
                    if message.is_empty() {
                        s.add_layer(Dialog::info(
                            "You must provide an message to sign!"
                        ));
                        return;
                    }
                    if let Ok(signature) = sign_message(&private_key, &message, &coin) {
                        s.pop_layer();
                        show_inbox(s);
                        s.add_layer(
                        Dialog::text(format!("Successfully signed message: {}", signature))
                            .button("Back", go_back)
                            .button("Copy signature", move |s| copy_to_clip(s, signature.clone()))
                        );
                    } else {
                        s.add_layer(
                            Dialog::info(StyledString::styled("Failed to sign message", RED)),
                        );
                        return;
                    }
                }))
                .child(Button::new("Back", show_inbox))),
        )
        .title("Sign message")).with_name("hideable"),
    );
}
