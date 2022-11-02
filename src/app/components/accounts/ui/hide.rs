use super::remove::remove_account;
use crate::app::{
    components::storage::save::save_to_storage, constants::colours::RED, userdata::UserData,
};
use cursive::{utils::markup::StyledString, views::Dialog, Cursive};

pub fn hide_account(s: &mut Cursive) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    if wallet.accounts.len() == 1 {
        s.add_layer(Dialog::info("You can't hide your final account!"));
        return;
    }
    remove_account(s);
    let save_res = save_to_storage(s);
    if save_res.is_err() {
        s.add_layer(
            Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                .title("Error saving wallets data."),
        );
    }
}
