use super::*;
use crate::util::constants::{DATA_DIR_PATH, MESSAGES_DIR_PATH, WALLETS_PATH};
use ::bincode;
use cursive::event::{Event, EventResult, EventTrigger, MouseEvent};
use rand::RngCore;

use crate::util::wallet::*;

fn add_account(s: &mut Cursive, index: Option<u32>, prefix: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &mut data.wallets[data.wallet_idx];
    let mut i = 0;
    let mut last = wallet.indexes[i];
    if let Some(index) = index {
        if index < last {
            wallet.indexes.insert(0, index);
            wallet
                .accounts
                .insert(0, Account::with_index(wallet, index, prefix));
            return;
        }
    } else {
        if last != 0 {
            wallet.indexes.insert(0, 0);
            wallet
                .accounts
                .insert(0, Account::with_index(wallet, 0, prefix));
            return;
        }
    }
    for idx in wallet.indexes[1..].iter() {
        if *idx != last + 1 {
            break;
        }
        i += 1;
        last = wallet.indexes[i]
    }
    if index.is_none() {
        wallet.indexes.insert(i + 1, last + 1);
        wallet
            .accounts
            .insert(i + 1, Account::with_index(wallet, last + 1, prefix));
    } else {
        wallet.indexes.push(index.unwrap());
        wallet
            .accounts
            .push(Account::with_index(wallet, index.unwrap(), prefix));
    }
}

pub fn load_wallets(s: &mut Cursive, data_path: PathBuf) {
    s.pop_layer();
    let wallets_file = data_path.join(WALLETS_PATH);
    if wallets_file.exists() {
        let encrypted_bytes = fs::read(&wallets_file).unwrap_or_else(|e| {
            let content = format!(
                "Failed to read {} file at path: {:?}\nError: {}",
                WALLETS_PATH, wallets_file, e
            );
            s.add_layer(Dialog::info(content));
            vec![]
        });
        if encrypted_bytes.is_empty() {
            show_wallets(s);
            return;
        }
        let data = &mut s.user_data::<UserData>().unwrap();
        data.encrypted_bytes = encrypted_bytes;
        s.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .child(DummyView)
                    .child(
                        EditView::new()
                            .secret()
                            .on_submit(move |s, password| {
                                load_with_password(s, password);
                            })
                            .with_name("password"),
                    )
                    .child(DummyView)
                    .child(Button::new("Submit", move |s| {
                        let password = s
                            .call_on_name("password", |view: &mut EditView| view.get_content())
                            .unwrap();
                        load_with_password(s, &password);
                    })),
            )
            .title("Enter dagchat password")
            .max_width(80),
        );
    } else {
        show_wallets(s);
    }
}

fn load_with_password(s: &mut Cursive, password: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let bytes = decrypt_bytes(&data.encrypted_bytes, &password);
    if bytes.is_err() {
        s.add_layer(Dialog::info("Password was incorrect."));
        return;
    }
    data.password = password.to_string();

    let wallets_and_lookup_res = bincode::deserialize(&bytes.unwrap()[..]);
    if wallets_and_lookup_res.is_err() {
        show_wallets(s);
        s.add_layer(Dialog::info(StyledString::styled(
            format!(
                "Error parsing {} file. File was either corrupted or edited outside of dagchat.",
                WALLETS_PATH
            ),
            RED,
        )));
    } else {
        let wallets_and_lookup: WalletsAndLookup = wallets_and_lookup_res.unwrap();
        data.wallets = bincode::deserialize(&wallets_and_lookup.wallets_bytes).unwrap();
        data.lookup = bincode::deserialize(&wallets_and_lookup.lookup_bytes).unwrap();
        show_wallets(s);
    }
}

fn write_wallets(encrypted_bytes: Vec<u8>) -> Result<(), String> {
    if let Some(data_dir) = dirs::data_dir() {
        let wallets_file = data_dir.join(DATA_DIR_PATH).join(WALLETS_PATH);
        let write_res = fs::write(&wallets_file, encrypted_bytes);
        if write_res.is_err() {
            return Err(format!(
                "Failed to write to {} file at path: {:?}\nError: {:?}",
                WALLETS_PATH,
                wallets_file,
                write_res.err()
            ));
        }
    }
    Ok(())
}

pub fn save_wallets(s: &mut Cursive) -> Result<(), String> {
    let data = &s.user_data::<UserData>().unwrap();
    if data.wallets.is_empty() && data.lookup.is_empty() {
        write_wallets(vec![])?;
        return Ok(());
    }
    let wallets_bytes = bincode::serialize(&data.wallets).unwrap();
    let lookup_bytes = bincode::serialize(&data.lookup).unwrap();
    let wallets_and_lookup = WalletsAndLookup {
        wallets_bytes,
        lookup_bytes,
    };
    let encoded: Vec<u8> = bincode::serialize(&wallets_and_lookup).unwrap();
    let encrypted_bytes = encrypt_bytes(&encoded, &data.password);
    write_wallets(encrypted_bytes)?;
    //eprintln!("Saved wallets with password: {}", data.password);
    Ok(())
}

pub fn show_wallets(s: &mut Cursive) {
    s.pop_layer();
    // Need to add change password button
    let buttons = LinearLayout::vertical()
        .child(Button::new("Import", |s| add_wallet(s)))
        .child(Button::new("Create", |s| new_wallet_name(s)))
        .child(DummyView)
        .child(Button::new("Backup", |s| backup_wallet(s)))
        .child(Button::new("Delete", |s| remove_wallet(s)))
        .child(DummyView)
        .child(Button::new("Back", |s| {
            s.pop_layer();
            show_title(s);
        }))
        .child(DummyView);

    let mut select = SelectView::<String>::new().on_submit(select_wallet);

    let mut i = 1;
    let data = &s.user_data::<UserData>().unwrap();
    for wallet in &data.wallets {
        let tag = format!("{}. {}", i, wallet.name);
        select.add_item_str(&tag);
        i += 1;
    }
    let select = OnEventView::new(select).on_pre_event_inner(EventTrigger::mouse(), |s, e| {
        if let &Event::Mouse {
            event: MouseEvent::WheelUp,
            ..
        } = e
        {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        } else if let &Event::Mouse {
            event: MouseEvent::WheelDown,
            ..
        } = e
        {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        } else {
            None
        }
    });
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical().child(DummyView).child(
                LinearLayout::horizontal()
                    .child(
                        Dialog::around(select.with_name("wallets").scrollable().max_height(6))
                            .padding_lrtb(1, 1, 0, 0)
                            .title("Wallets"),
                    )
                    .child(DummyView)
                    .child(DummyView)
                    .child(buttons),
            ),
        )
        .title("Select a Wallet"),
    );
}

fn select_account(s: &mut Cursive, _: &str) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("accounts")
        .unwrap();
    let select = eventview.get_inner();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No account selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &mut data.wallets[data.wallet_idx];
    wallet.acc_idx = focus;
    load_current_account(s);
}

fn load_current_account(s: &mut Cursive) {
    let messages = messages::load_messages(s);
    let data = &mut s.user_data::<UserData>().unwrap();
    //eprintln!("Loaded messages: {:?}", messages);
    let wallet = &mut data.wallets[data.wallet_idx];
    wallet.accounts[wallet.acc_idx].messages = messages;
    receive::load_receivables(s);
}

fn select_wallet(s: &mut Cursive, _: &str) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("wallets")
        .unwrap();
    let select = eventview.get_inner();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No wallet selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let data = &mut s.user_data::<UserData>().unwrap();
    data.wallet_idx = focus;

    // Generate accounts for saved indexes
    let mut accounts: Vec<Account> = vec![];
    //eprintln!("Saved indexes: {:?}", data.wallets[focus].indexes);
    for index in &data.wallets[focus].indexes {
        accounts.push(Account::with_index(
            &data.wallets[focus],
            *index,
            &data.coin.prefix,
        ));
    }
    data.wallets[focus].accounts = accounts;
    show_accounts(s);
}

pub fn show_accounts(s: &mut Cursive) {
    s.pop_layer();
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[data.wallet_idx];
    let wallet_name = wallet.name.clone();
    let prefix = data.coin.prefix.clone();

    let mut buttons = LinearLayout::horizontal().child(DummyView);
    if !wallet.mnemonic.is_empty() {
        buttons.add_child(Button::new("Show next", move |s| {
            add_account(s, None, &prefix);
            let save_res = save_wallets(s);
            s.pop_layer();
            show_accounts(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving wallets data"),
                );
            }
        }));
        buttons.add_child(DummyView);
        buttons.add_child(Button::new("Show index", |s| add_index(s)));
        buttons.add_child(DummyView);
        buttons.add_child(Button::new("Hide", |s| {
            let data = &mut s.user_data::<UserData>().unwrap();
            let wallet = &data.wallets[data.wallet_idx];
            if wallet.accounts.len() == 1 {
                s.add_layer(Dialog::info("You can't hide your final account!"));
                return;
            }
            remove_account(s);
            let save_res = save_wallets(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving wallets data"),
                );
            }
        }));
        buttons.add_child(DummyView);
    }
    buttons.add_child(Button::new("Back", |s| {
        s.pop_layer();
        show_wallets(s);
    }));

    let mut select = SelectView::<String>::new().on_submit(select_account);

    for account in &data.wallets[data.wallet_idx].accounts {
        let tag = format!("{}: {}", account.index, account.address);
        select.add_item_str(&tag)
    }

    let select = OnEventView::new(select).on_pre_event_inner(EventTrigger::mouse(), |s, e| {
        if let &Event::Mouse {
            event: MouseEvent::WheelUp,
            ..
        } = e
        {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        } else if let &Event::Mouse {
            event: MouseEvent::WheelDown,
            ..
        } = e
        {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        } else {
            None
        }
    });
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical().child(DummyView).child(
                LinearLayout::horizontal().child(
                    LinearLayout::vertical()
                        .child(buttons)
                        .child(DummyView)
                        .child(
                            Dialog::around(
                                select
                                    .with_name("accounts")
                                    .scrollable()
                                    .max_width(38)
                                    .max_height(5),
                            )
                            .padding_lrtb(1, 1, 0, 0)
                            .title("Accounts"),
                        ),
                ),
            ),
        )
        .title(wallet_name),
    );
}

fn add_index(s: &mut Cursive) {
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(TextView::new("Account index (0 - 4,294,967,295)"))
                .child(EditView::new().on_submit(process_idx).with_name("index")),
        )
        .h_align(HAlign::Center)
        .button("Submit", move |s| {
            let idx = s
                .call_on_name("index", |view: &mut EditView| view.get_content())
                .unwrap();
            process_idx(s, &idx);
        })
        .button("Back", |s| {
            s.pop_layer();
        })
        .title("Show account with index"),
    );
}

fn process_idx(s: &mut Cursive, idx: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let prefix = &data.coin.prefix.clone();
    let wallet = &data.wallets[data.wallet_idx];
    let index_res: Result<u32, _> = idx.parse();
    if index_res.is_err() {
        s.add_layer(Dialog::info(
            "Error: index was not an integer within the valid range.",
        ));
        return;
    } else if wallet.indexes.contains(index_res.as_ref().unwrap()) {
        s.add_layer(Dialog::info("This account has already been added!"));
        return;
    } else {
        add_account(s, Some(index_res.unwrap()), prefix);
        let save_res = save_wallets(s);
        s.pop_layer();
        s.pop_layer();
        show_accounts(s);
        if save_res.is_err() {
            s.add_layer(
                Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                    .title("Error saving wallets data"),
            );
        }
    }
}

fn backup_wallet(s: &mut Cursive) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("wallets")
        .unwrap();
    let select = eventview.get_inner();
    let selected_idx;
    match select.selected_id() {
        None => {
            s.add_layer(Dialog::info("No wallet selected."));
            return;
        }
        Some(focus) => {
            selected_idx = focus;
        }
    }
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &data.wallets[selected_idx];
    let mut content = Dialog::around(LinearLayout::vertical().child(DummyView).child(
        TextView::new(StyledString::styled(
            "Make sure you are in a safe location before viewing your mnemonic, seed or key.",
            RED,
        )),
    ))
    .h_align(HAlign::Center)
    .title("Backup wallet");
    if &wallet.mnemonic != "" {
        let mnemonic = wallet.mnemonic.clone();
        content.add_button("Mnemonic", move |s| {
            let mnemonic = mnemonic.clone();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(DummyView)
                        .child(TextView::new(&mnemonic)),
                )
                .h_align(HAlign::Center)
                .button("Copy", move |s| {
                    s.pop_layer();
                    s.pop_layer();
                    copy_to_clip(s, mnemonic.clone())
                })
                .button("Back", |s| go_back(s))
                .title("Mnemonic")
                .max_width(80),
            );
        });
        let seed = hex::encode(wallet.seed);
        content.add_button("Hex seed", move |s| {
            let seed = seed.clone();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(DummyView)
                        .child(TextView::new(&seed)),
                )
                .h_align(HAlign::Center)
                .button("Copy", move |s| {
                    s.pop_layer();
                    s.pop_layer();
                    copy_to_clip(s, seed.clone())
                })
                .button("Back", |s| go_back(s))
                .title("Seed"),
            );
        });
    } else {
        let private_key = hex::encode(wallet.accounts[wallet.acc_idx].private_key);
        content.add_button("Private key", move |s| {
            let private_key = private_key.clone();
            s.add_layer(
                Dialog::around(
                    LinearLayout::vertical()
                        .child(DummyView)
                        .child(TextView::new(&private_key)),
                )
                .h_align(HAlign::Center)
                .button("Copy", move |s| {
                    s.pop_layer();
                    s.pop_layer();
                    copy_to_clip(s, private_key.clone())
                })
                .button("Back", |s| go_back(s))
                .title("Private key"),
            );
        });
    }
    content.add_button("Back", |s| go_back(s));

    s.add_layer(content.max_width(80));
}

fn remove_account(s: &mut Cursive) {
    let mut eventview = s
        .find_name::<OnEventView<SelectView<String>>>("accounts")
        .unwrap();
    let select = eventview.get_inner_mut();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No account selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let data = &mut s.user_data::<UserData>().unwrap();
    let wallet = &mut data.wallets[data.wallet_idx];
    wallet.accounts.remove(focus);
    wallet.indexes.remove(focus);
    select.remove_item(focus);
}

fn remove_wallet(s: &mut Cursive) {
    let eventview = s
        .find_name::<OnEventView<SelectView<String>>>("wallets")
        .unwrap();
    let select = eventview.get_inner();
    let focus_opt = select.selected_id();
    if focus_opt.is_none() {
        s.add_layer(Dialog::info("No wallet selected."));
        return;
    }
    let focus = focus_opt.unwrap();
    let warning = StyledString::styled(
        "If you have not backed up this wallet, all of its accounts will be lost forever.",
        RED,
    );
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(TextView::new(warning))
                .child(DummyView),
        )
        .h_align(HAlign::Center)
        .button("Back", |s| {
            s.pop_layer();
        })
        .button("Backup", |s| backup_wallet(s))
        .button("Confirm", move |s| {
            let data = &mut s.user_data::<UserData>().unwrap();
            let wallet = &data.wallets[focus];

            // Remove account addresses from lookup if they
            // have no messages linked.
            for account in &wallet.accounts {
                let data_dir = dirs::data_dir().unwrap();
                let messages_dir = data_dir.join(DATA_DIR_PATH).join(MESSAGES_DIR_PATH);
                if data.lookup.contains_key(&account.address) {
                    let filename =
                        format!("{}.dagchat", data.lookup.get(&account.address).unwrap());
                    let messages_file = messages_dir.join(filename);
                    if !messages_file.exists() {
                        data.lookup.remove(&account.address);
                    }
                }
            }
            data.wallets.remove(focus);
            let save_res = save_wallets(s);
            s.pop_layer();
            s.pop_layer();
            show_wallets(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving wallets data"),
                );
            }
        })
        .title("Confirm wallet deletion"),
    );
}

fn add_wallet(s: &mut Cursive) {
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

fn get_name(s: &mut Cursive) -> String {
    let name = s
        .call_on_name("name", |view: &mut EditView| view.get_content())
        .unwrap();
    s.pop_layer();
    name.to_string()
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
                    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                    let clip = clipboard
                        .get_contents()
                        .unwrap_or_else(|_| String::from("Failed to read clipboard."));
                    view.set_content(clip);
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
                    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
                    let clip = clipboard
                        .get_contents()
                        .unwrap_or_else(|_| String::from("Failed to read clipboard."));
                    view.set_content(clip);
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

fn new_wallet_name(s: &mut Cursive) {
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

fn new_wallet(s: &mut Cursive, name: &str) {
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

fn set_password<F: 'static>(s: &mut Cursive, on_success: F)
where
    F: Fn(&mut Cursive),
{
    let warning = StyledString::styled(
        "Always backup or write down your mnemonics, seeds or keys elsewhere in case you forget your password.", RED);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(DummyView)
                .child(TextView::new("Enter password"))
                .child(EditView::new().secret().with_name("password"))
                .child(DummyView)
                .child(TextView::new("Confirm password"))
                .child(EditView::new().secret().with_name("confirm"))
                .child(DummyView)
                .child(TextView::new(warning)),
        )
        .h_align(HAlign::Center)
        .button("Submit", move |s| {
            let password = s
                .call_on_name("password", |view: &mut EditView| view.get_content())
                .unwrap();
            let confirmed = s
                .call_on_name("confirm", |view: &mut EditView| view.get_content())
                .unwrap();
            if password.is_empty() {
                s.add_layer(Dialog::info("Password can't be blank."));
                return;
            }
            if password != confirmed {
                s.add_layer(Dialog::info("Passwords did not match."));
                return;
            }
            let msg_save_res = messages::change_message_passwords(s, &password);
            let data = &mut s.user_data::<UserData>().unwrap();
            data.password = password.to_string();
            let acc_save_res = save_wallets(s);
            s.pop_layer();
            if acc_save_res.is_ok() && msg_save_res.is_ok() {
                on_success(s);
            } else if acc_save_res.is_err() {
                show_wallets(s);
                s.add_layer(Dialog::info(StyledString::styled(acc_save_res.err().unwrap(),
                    RED,
                )).title("Fatal error saving wallets"));
            } else if msg_save_res.is_err() {
                show_wallets(s);
                s.add_layer(Dialog::info(StyledString::styled(msg_save_res.err().unwrap(),
                    RED,
                )).title("Fatal error saving messages"));
            }
        })
        .button("Info", |s| {
            let content = "\nThe password you are setting up for dagchat is used to encrypt your wallets, messages (If 'Encrypt and save' messages setting is selected) and address book when they are saved on your device. It should be strong and contain a range of characters (UPPERCASE, lowercase, numb3rs and symbo!s). Without this password, dagchat will not be able to decrypt any of your saved wallets, messages or address book.";
            s.add_layer(Dialog::info(content).title("What is this password?").max_width(80));
        })
        .title("Create a password for dagchat")
        .max_width(80),
    );
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
