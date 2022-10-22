use super::super::{save::save_wallets, structs::Wallet};
use super::primary::show_wallets;
use crate::app::components::{
    accounts::ui::select::load_current_account, password::ui::primary::set_password,
};
use crate::app::{
    clipboard::*,
    constants::colours::{OFF_WHITE, RED},
    helpers::get_name,
    userdata::UserData,
};
use crate::crypto::mnemonic::{seed_to_mnemonic, validate_mnemonic};
use cursive::views::{Dialog, DummyView, EditView, LinearLayout, TextView};
use cursive::{align::HAlign, traits::*, utils::markup::StyledString, Cursive};
use rand::RngCore;

pub fn add_wallet(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let coin = &data.coin.name;
    let colour = data.coin.colour;

    let name_input = EditView::new()
        .content(format!("Default {}", data.wallets.len() + 1))
        .with_name("name");
    let import_msg = format!("Choose a way to import your {} wallet.", coin);

    let content = LinearLayout::vertical()
        .child(DummyView)
        .child(TextView::new(StyledString::styled("Wallet name", colour)))
        .child(name_input)
        .child(DummyView)
        .child(TextView::new(StyledString::styled(import_msg, colour)));
    s.add_layer(
        Dialog::around(content)
            .h_align(HAlign::Center)
            .button("Mnemonic", |s| {
                let name = get_name(s);
                show_from_mnemonic(s, name);
            })
            .button("Seed", |s| {
                let name = get_name(s);
                from_seedorkey(s, String::from("seed"), name);
            })
            .button("Private Key", |s| {
                let name = get_name(s);
                from_seedorkey(s, String::from("private key"), name);
            })
            .button("Back", |s| show_wallets(s))
            .title("Import wallet"),
    );
}

fn show_from_mnemonic(s: &mut Cursive, name: String) {
    s.pop_layer();
    let on_submit_name = name.clone();
    s.add_layer(
        Dialog::new()
            .title("Enter your 24 word mnemonic")
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    .on_submit(move |s, mnemonic| {
                        process_from_mnemonic(s, mnemonic, on_submit_name.clone())
                    })
                    .with_name("mnemonic")
                    .fixed_width(29),
            )
            .h_align(HAlign::Center)
            .button("Done", move |s| {
                let mnemonic = s
                    .call_on_name("mnemonic", |view: &mut EditView| view.get_content())
                    .unwrap();
                process_from_mnemonic(s, &mnemonic, name.clone());
            })
            .button("Paste", |s| {
                s.call_on_name("mnemonic", |view: &mut EditView| {
                    view.set_content(paste_clip());
                })
                .unwrap();
            })
            .button("Back", |s| {
                show_wallets(s);
            }),
    );
}

fn process_from_mnemonic(s: &mut Cursive, mnemonic: &str, name: String) {
    let seed = validate_mnemonic(&mnemonic);
    let content;
    s.pop_layer();
    if !mnemonic.is_empty() && seed.is_some() {
        let seed_bytes = seed.unwrap();
        let data = &s.user_data::<UserData>().unwrap();
        let wallet = Wallet::new(mnemonic.to_string(), seed_bytes, name, &data.coin.prefix);
        setup_wallet(s, wallet, |s| {
            import_success(s, "Successfully imported wallet from mnemonic phrase.")
        });
    } else {
        content = "The mnemonic you entered was not valid.";
        s.add_layer(
            Dialog::around(TextView::new(content))
                .button("Back", move |s| show_from_mnemonic(s, name.clone())),
        );
        return;
    }
}

fn from_seedorkey(s: &mut Cursive, seed_or_key: String, name: String) {
    s.pop_layer();
    let on_submit_seed_or_key = seed_or_key.clone();
    let on_submit_name = name.clone();
    s.add_layer(
        Dialog::new()
            .title(format!("Enter your {}", seed_or_key))
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    .on_submit(move |s, sork_raw| {
                        process_from_seedorkey(
                            s,
                            sork_raw.to_string(),
                            &on_submit_seed_or_key,
                            on_submit_name.clone(),
                        );
                    })
                    .with_name("seedorkey")
                    .fixed_width(29),
            )
            .h_align(HAlign::Center)
            .button("Done", move |s| {
                let sork_raw = s
                    .call_on_name("seedorkey", |view: &mut EditView| view.get_content())
                    .unwrap();
                process_from_seedorkey(s, sork_raw.to_string(), &seed_or_key, name.clone());
            })
            .button("Paste", |s| {
                s.call_on_name("seedorkey", |view: &mut EditView| {
                    view.set_content(paste_clip());
                })
                .unwrap();
            })
            .button("Back", |s| {
                show_wallets(s);
            }),
    );
}

fn process_from_seedorkey(s: &mut Cursive, sork_raw: String, seed_or_key: &String, name: String) {
    let sork_val = sork_raw.trim();
    if sork_val.len() != 64 {
        s.add_layer(Dialog::info(format!(
            "Error: {} was invalid - not 64 characters long.",
            seed_or_key
        )));
        return;
    }
    let bytes_opt = hex::decode(sork_val);
    if bytes_opt.is_err() {
        s.add_layer(Dialog::info(format!(
            "Error: {} was invalid - failed to decode hex.",
            seed_or_key
        )));
        return;
    }
    let bytes = bytes_opt.unwrap();
    let sork_bytes: [u8; 32] = bytes.try_into().unwrap();
    let data = &s.user_data::<UserData>().unwrap();
    let wallet: Wallet;
    if seed_or_key == "seed" {
        let mnemonic = seed_to_mnemonic(&sork_bytes);
        wallet = Wallet::new(mnemonic, sork_bytes, name, &data.coin.prefix);
    } else {
        wallet = Wallet::new_key(sork_bytes, name, &data.coin.prefix);
    }
    let content = format!("Successfully imported wallet from {}.", seed_or_key);
    setup_wallet(s, wallet, move |s| import_success(s, &content));
}

pub fn new_wallet_name(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let colour = data.coin.colour;
    let name_input = EditView::new()
        .on_submit(new_wallet)
        .content(format!("Default {}", data.wallets.len() + 1))
        .with_name("name");

    let content = LinearLayout::vertical()
        .child(DummyView)
        .child(TextView::new(StyledString::styled("Wallet name", colour)))
        .child(name_input);

    s.add_layer(
        Dialog::around(content)
            .button("Done", |s| {
                let name = s
                    .call_on_name("name", |view: &mut EditView| view.get_content())
                    .unwrap();
                new_wallet(s, &name);
            })
            .title("Set up new wallet"),
    );
}

pub fn new_wallet(s: &mut Cursive, name: &str) {
    s.pop_layer();
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let mut csprng = rand::thread_rng();
    let mut seed_bytes = [0u8; 32];
    csprng.fill_bytes(&mut seed_bytes);
    let mnemonic = seed_to_mnemonic(&seed_bytes);
    let wallet = Wallet::new(
        mnemonic.clone(),
        seed_bytes,
        name.to_string(),
        &data.coin.prefix,
    );
    setup_wallet(s, wallet, move |s| {
        create_success(s, mnemonic.clone(), hex::encode(seed_bytes))
    });
}
fn setup_wallet<F: 'static>(s: &mut Cursive, wallet: Wallet, on_success: F)
where
    F: Fn(&mut Cursive),
{
    let data = &mut s.user_data::<UserData>().unwrap();
    data.wallets.push(wallet);
    data.wallet_idx = data.wallets.len() - 1;

    let data = &mut s.user_data::<UserData>().unwrap();
    if data.wallets.len() == 1 && data.password.is_empty() {
        set_password(s, on_success);
    } else {
        let save_res = save_wallets(s);
        if save_res.is_ok() {
            on_success(s);
        } else {
            show_wallets(s);
            s.add_layer(
                Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                    .title("Error saving wallets data"),
            );
        }
    }
}

fn import_success(s: &mut Cursive, content: &str) {
    s.add_layer(
        Dialog::around(TextView::new(content).max_width(80))
            .button("Load", |s| load_current_account(s))
            .button("Back", |s| {
                s.pop_layer();
                show_wallets(s);
            }),
    );
}

fn create_success(s: &mut Cursive, mnemonic: String, seed: String) {
    s.pop_layer();
    let data = &mut s.user_data::<UserData>().unwrap();
    let mut content = StyledString::styled("\nMnemonic\n", data.coin.colour);
    content.append(StyledString::styled(&mnemonic, OFF_WHITE));
    content.append(StyledString::styled("\n\nSeed\n", data.coin.colour));
    content.append(StyledString::styled(&seed, OFF_WHITE));
    s.add_layer(
        Dialog::around(TextView::new(content).max_width(80))
            .h_align(HAlign::Center)
            .button("Load", |s| load_current_account(s))
            .button("Back", |s| show_wallets(s))
            .button("Copy mnemonic", move |s| copy_to_clip(s, mnemonic.clone()))
            .button("Copy seed", move |s| copy_to_clip(s, seed.clone()))
            .title("Successfully generated new wallet")
            .max_width(80),
    );
}
