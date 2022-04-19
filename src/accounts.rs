use super::*;

use rand::RngCore;

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountsAndLookup {
    accounts_bytes: Vec<u8>,
    lookup_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub mnemonic: Option<String>,
    pub seed: Option<[u8; 32]>,
    pub key_idx: u32,
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub address: String,
    pub balance: u128,
    pub receivables: Vec<Receivable>,
}

impl Account {
    fn from_mnemonic(mnemonic: Option<String>, seed: Option<[u8; 32]>, prefix: &str) -> Account {
        let (private_key, public_key) = Account::get_keypair(&seed.unwrap());
        Account {
            mnemonic,
            seed,
            key_idx: 0,
            private_key,
            public_key,
            address: dcutil::get_address(&public_key, prefix),
            balance: 0,
            receivables: vec![],
        }
    }
    fn from_seed(seed: Option<[u8; 32]>, prefix: &str) -> Account {
        let (private_key, public_key) = Account::get_keypair(&seed.unwrap());
        Account {
            mnemonic: None,
            seed,
            key_idx: 0,
            private_key,
            public_key,
            address: dcutil::get_address(&public_key, prefix),
            balance: 0,
            receivables: vec![],
        }
    }

    fn from_private_key(private_key: [u8; 32], prefix: &str) -> Account {
        let public_key = Account::get_public_key(&private_key);
        Account {
            mnemonic: None,
            seed: None,
            key_idx: 0,
            private_key,
            public_key,
            address: dcutil::get_address(&public_key, prefix),
            balance: 0,
            receivables: vec![],
        }
    }

    fn get_keypair(seed: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
        let private_key = dcutil::get_private_key(seed, 0);
        let public_key = Account::get_public_key(&private_key);
        (private_key, public_key)
    }
    fn get_public_key(private_key: &[u8; 32]) -> [u8; 32] {
        let dalek = ed25519_dalek::SecretKey::from_bytes(private_key).unwrap();
        let public_key = ed25519_dalek::PublicKey::from(&dalek);
        public_key.to_bytes()
    }
}

pub fn load_accounts(s: &mut Cursive, data_path: PathBuf) {
    s.pop_layer();
    let accounts_file = data_path.join(ACCOUNTS_PATH);
    if accounts_file.exists() {
        let encrypted_bytes = fs::read(&accounts_file).unwrap_or_else(|e| {
            let content = format!(
                "Failed to read {} file at path: {:?}\nError: {}",
                ACCOUNTS_PATH, accounts_file, e
            );
            s.add_layer(Dialog::info(content));
            vec![]
        });
        if encrypted_bytes.is_empty() {
            show_accounts(s);
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
        show_accounts(s);
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

    let accounts_and_lookup_res = bincode::deserialize(&bytes.unwrap()[..]);
    if accounts_and_lookup_res.is_err() {
        show_accounts(s);
        s.add_layer(Dialog::info(StyledString::styled(
            format!(
                "Error parsing {} file. File was either corrupted or edited outside of dagchat.",
                ACCOUNTS_PATH
            ),
            RED,
        )));
    } else {
        let accounts_and_lookup: AccountsAndLookup = accounts_and_lookup_res.unwrap();
        data.accounts = bincode::deserialize(&accounts_and_lookup.accounts_bytes).unwrap();
        data.lookup = bincode::deserialize(&accounts_and_lookup.lookup_bytes).unwrap();
        show_accounts(s);
    }
}

fn write_accounts(encrypted_bytes: Vec<u8>) -> Result<(), String> {
    if let Some(data_dir) = dirs::data_dir() {
        let accounts_file = data_dir.join(DATA_DIR_PATH).join(ACCOUNTS_PATH);
        let write_res = fs::write(&accounts_file, encrypted_bytes);
        if write_res.is_err() {
            return Err(format!(
                "Failed to write to {} file at path: {:?}\nError: {:?}",
                ACCOUNTS_PATH,
                accounts_file,
                write_res.err()
            ));
        }
    }
    Ok(())
}

pub fn save_accounts(s: &mut Cursive) -> Result<(), String> {
    let data = &s.user_data::<UserData>().unwrap();
    if data.accounts.is_empty() && data.lookup.is_empty() {
        write_accounts(vec![])?;
        return Ok(());
    }
    let accounts_bytes = bincode::serialize(&data.accounts).unwrap();
    let lookup_bytes = bincode::serialize(&data.lookup).unwrap();
    let accounts_and_lookup = AccountsAndLookup {
        accounts_bytes,
        lookup_bytes,
    };
    let encoded: Vec<u8> = bincode::serialize(&accounts_and_lookup).unwrap();
    let encrypted_bytes = encrypt_bytes(&encoded, &data.password);
    write_accounts(encrypted_bytes)?;
    eprintln!("Saved accounts with password: {}", data.password);
    Ok(())
}

pub fn show_accounts(s: &mut Cursive) {
    s.pop_layer();
    let data: UserData = s.take_user_data().unwrap();
    // Need to add change password button
    let buttons = LinearLayout::vertical()
        .child(Button::new("Import", |s| add_account(s)))
        .child(Button::new("Create", |s| new_account(s)))
        .child(DummyView)
        .child(Button::new("Back", |s| {
            s.pop_layer();
            show_title(s);
        }))
        .child(DummyView);

    let select = SelectView::<String>::new()
        .on_submit(select_account)
        .with_name("accounts")
        .scrollable()
        .max_height(5);

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical().child(DummyView).child(
                LinearLayout::horizontal()
                    .child(
                        Dialog::around(select)
                            .padding_lrtb(1, 1, 0, 0)
                            .title("Accounts"),
                    )
                    .child(DummyView)
                    .child(DummyView)
                    .child(buttons),
            ),
        )
        .title("Select an Account"),
    );

    let mut i = 1;
    for account in &data.accounts {
        let tag;
        if account.mnemonic.is_some() {
            tag = format!("{}. From mnemonic", i);
        } else if account.seed.is_some() {
            tag = format!("{}. From seed", i);
        } else {
            tag = format!("{}. From private key", i);
        }
        s.call_on_name("accounts", |view: &mut SelectView<String>| {
            view.add_item_str(&tag)
        });
        i += 1;
    }

    s.set_user_data(data);
}

fn select_account(s: &mut Cursive, _: &str) {
    let select = s.find_name::<SelectView<String>>("accounts").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No account selected.")),
        Some(focus) => {
            let data = &mut s.user_data::<UserData>().unwrap();
            data.acc_idx = focus;

            // Seemlessly use accounts between networks
            if !data.accounts[focus].address.starts_with(&data.coin.prefix) {
                let public_key = &data.accounts[focus].public_key;
                data.accounts[focus].address = get_address(public_key, &data.coin.prefix);
            }

            let account = &data.accounts[focus];
            let origin;
            let mut extra = format!(" (at index {})", account.key_idx);

            let mut outer = Dialog::new()
                .h_align(HAlign::Center)
                .button("Load", move |s| {
                    s.pop_layer();
                    load_current_account(s);
                })
                .button("Back", |s| {
                    s.pop_layer();
                    s.pop_layer();
                    show_accounts(s)
                });
            if account.mnemonic.is_some() {
                origin = "mnemonic";
                add_idx_button(&mut outer);
            } else if account.seed.is_some() {
                origin = "seed";
                add_idx_button(&mut outer);
            } else {
                origin = "private key";
                extra = String::from("");
            }
            outer.add_button("Backup", move |s| backup_account(s, origin));
            outer.add_button("Remove", move |s| remove_account(s, focus));
            let content = LinearLayout::vertical()
                .child(DummyView)
                .child(TextView::new(StyledString::styled(
                    format!("Account address{}", extra),
                    OFF_WHITE,
                )))
                .child(TextView::new(StyledString::styled(
                    &account.address,
                    data.coin.colour,
                )))
                .child(DummyView)
                .child(TextView::new(StyledString::styled(
                    "Account type",
                    OFF_WHITE,
                )))
                .child(TextView::new(StyledString::styled(
                    format!("loaded from {}", origin),
                    data.coin.colour,
                )));
            s.add_layer(
                outer
                    .content(content)
                    .title(format!("Account {}", focus + 1)),
            );
        }
    }
}

fn load_current_account(s: &mut Cursive) {
    let messages = messages::load_messages(s);
    let data = &mut s.user_data::<UserData>().unwrap();
    eprintln!("Loaded messages: {:?}", messages);
    data.acc_messages = messages;
    receive::load_receivables(s);
}
fn backup_account(s: &mut Cursive, origin: &str) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let account = &data.accounts[data.acc_idx];
    let mut content = Dialog::around(LinearLayout::vertical().child(DummyView).child(
        TextView::new(StyledString::styled(
            "Make sure you are in a safe location before viewing your mnemonic, seed or key.",
            RED,
        )),
    ))
    .h_align(HAlign::Center)
    .title("Backup account");

    if origin == "mnemonic" {
        let mnemonic = account.mnemonic.as_ref().unwrap().clone();
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
    }
    if origin == "mnemonic" || origin == "seed" {
        let seed = hex::encode(account.seed.unwrap());
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
        let private_key = hex::encode(account.private_key);
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

fn add_idx_button(dialog: &mut Dialog) {
    dialog.add_button("Index", move |s| {
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
            .title("Change account index"),
        )
    });
}

fn process_idx(s: &mut Cursive, idx: &str) {
    let index_res: Result<u32, _> = idx.parse();
    if index_res.is_err() {
        s.add_layer(Dialog::info(
            "Error: index was not an integer within the valid range.",
        ));
        return;
    } else {
        let index: u32 = index_res.unwrap();
        let data = &mut s.user_data::<UserData>().unwrap();
        let mut account = &mut data.accounts[data.acc_idx];
        account.key_idx = index;
        let seed = account.seed.unwrap();
        account.private_key = get_private_key(&seed, index);
        account.public_key = Account::get_public_key(&account.private_key);
        account.address = get_address(&account.public_key, &data.coin.prefix);
        s.pop_layer();
        s.pop_layer();
        select_account(s, "");
    }
}

fn remove_account(s: &mut Cursive, idx: usize) {
    let warning = StyledString::styled(
        "If you have not backed up this account, it will be lost forever.",
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
        .button("Confirm", move |s| {
            let data = &mut s.user_data::<UserData>().unwrap();
            let account = &data.accounts[idx];
            if data.lookup.contains_key(&account.address) {
                let data_dir = dirs::data_dir().unwrap();
                let messages_dir = data_dir.join(DATA_DIR_PATH).join(MESSAGES_DIR_PATH);
                let filename = format!("{}.dagchat", data.lookup.get(&account.address).unwrap());
                let messages_file = messages_dir.join(filename);
                if !messages_file.exists() {
                    data.lookup.remove(&account.address);
                }
            }
            data.accounts.remove(idx);
            let save_res = save_accounts(s);
            s.pop_layer();
            s.pop_layer();
            s.pop_layer();
            show_accounts(s);
            if save_res.is_err() {
                s.add_layer(
                    Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                        .title("Error saving accounts data"),
                );
            }
        })
        .title("Confirm account deletion"),
    );
}

fn add_account(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let coin = &data.coin.name;
    let colour = data.coin.colour;
    let content = format!("Choose a way to import your {} wallet.", coin);

    let content = LinearLayout::vertical()
        .child(DummyView)
        .child(TextView::new(StyledString::styled(content, colour)))
        .child(DummyView)
        .child(Button::new("Mnemonic", |s| show_from_mnemonic(s)))
        .child(Button::new("Hex Seed", |s| {
            from_seedorkey(s, String::from("seed"))
        }))
        .child(Button::new("Private Key", |s| {
            from_seedorkey(s, String::from("private key"))
        }))
        .child(DummyView)
        .child(Button::new("Back", |s| show_accounts(s)));
    s.add_layer(Dialog::around(content).title("Import account"));
}

fn show_from_mnemonic(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .title("Enter your 24 word mnemonic")
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    .on_submit(process_from_mnemonic)
                    .with_name("mnemonic")
                    .fixed_width(29),
            )
            .h_align(HAlign::Center)
            .button("Done", |s| {
                let mnemonic = s
                    .call_on_name("mnemonic", |view: &mut EditView| view.get_content())
                    .unwrap();
                process_from_mnemonic(s, &mnemonic);
            })
            .button("Paste", |s| {
                s.call_on_name("mnemonic", |view: &mut EditView| {
                    let mut clipboard = Clipboard::new().unwrap();
                    let clip = clipboard
                        .get_text()
                        .unwrap_or_else(|_| String::from("Failed to read clipboard."));
                    view.set_content(clip);
                })
                .unwrap();
            })
            .button("Back", |s| {
                show_accounts(s);
            }),
    );
}

fn process_from_mnemonic(s: &mut Cursive, mnemonic: &str) {
    let seed = validate_mnemonic(&mnemonic);
    let content;
    s.pop_layer();
    if !mnemonic.is_empty() && seed.is_some() {
        let seed_bytes = seed.unwrap();
        let data = &s.user_data::<UserData>().unwrap();
        let account = Account::from_mnemonic(
            Some(mnemonic.to_string()),
            Some(seed_bytes),
            &data.coin.prefix,
        );
        setup_account(s, account, |s| {
            import_success(s, "Successfully imported account from mnemonic phrase.")
        });
    } else {
        content = "The mnemonic you entered was not valid.";
        s.add_layer(
            Dialog::around(TextView::new(content)).button("Back", |s| show_from_mnemonic(s)),
        );
        return;
    }
}
fn from_seedorkey(s: &mut Cursive, seed_or_key: String) {
    s.pop_layer();
    let on_submit_seed_or_key = seed_or_key.clone();
    s.add_layer(
        Dialog::new()
            .title(format!("Enter your {}", seed_or_key))
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    .on_submit(move |s, sork_raw| {
                        process_from_seedorkey(s, sork_raw.to_string(), &on_submit_seed_or_key);
                    })
                    .with_name("seedorkey")
                    .fixed_width(29),
            )
            .h_align(HAlign::Center)
            .button("Done", move |s| {
                let sork_raw = s
                    .call_on_name("seedorkey", |view: &mut EditView| view.get_content())
                    .unwrap();
                process_from_seedorkey(s, sork_raw.to_string(), &seed_or_key);
            })
            .button("Paste", |s| {
                s.call_on_name("seedorkey", |view: &mut EditView| {
                    let mut clipboard = Clipboard::new().unwrap();
                    let clip = clipboard
                        .get_text()
                        .unwrap_or_else(|_| String::from("Failed to read clipboard."));
                    view.set_content(clip);
                })
                .unwrap();
            })
            .button("Back", |s| {
                show_accounts(s);
            }),
    );
}

fn process_from_seedorkey(s: &mut Cursive, sork_raw: String, seed_or_key: &String) {
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
    let account: Account;
    if seed_or_key == "seed" {
        account = Account::from_seed(Some(sork_bytes), &data.coin.prefix);
    } else {
        account = Account::from_private_key(sork_bytes, &data.coin.prefix);
    }
    let content = format!("Successfully imported account from {}.", seed_or_key);
    setup_account(s, account, move |s| import_success(s, &content));
}

fn new_account(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let mut csprng = rand::thread_rng();
    let mut seed_bytes = [0u8; 32];
    csprng.fill_bytes(&mut seed_bytes);
    let mnemonic = seed_to_mnemonic(&seed_bytes);
    let account =
        Account::from_mnemonic(Some(mnemonic.clone()), Some(seed_bytes), &data.coin.prefix);
    setup_account(s, account, move |s| {
        create_success(s, mnemonic.clone(), hex::encode(seed_bytes))
    });
}

fn setup_account<F: 'static>(s: &mut Cursive, account: Account, on_success: F)
where
    F: Fn(&mut Cursive),
{
    let data = &mut s.user_data::<UserData>().unwrap();
    data.accounts.push(account);
    data.acc_idx = data.accounts.len() - 1;

    let data = &mut s.user_data::<UserData>().unwrap();
    if data.accounts.len() == 1 && data.password.is_empty() {
        set_password(s, on_success);
    } else {
        let save_res = save_accounts(s);
        if save_res.is_ok() {
            on_success(s);
        } else {
            show_accounts(s);
            s.add_layer(
                Dialog::info(StyledString::styled(save_res.err().unwrap(), RED))
                    .title("Error saving accounts data"),
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
            let acc_save_res = save_accounts(s);
            s.pop_layer();
            if acc_save_res.is_ok() && msg_save_res.is_ok() {
                on_success(s);
            } else if acc_save_res.is_err() {
                show_accounts(s);
                s.add_layer(Dialog::info(StyledString::styled(acc_save_res.err().unwrap(),
                    RED,
                )).title("Fatal error saving accounts"));
            } else if msg_save_res.is_err() {
                show_accounts(s);
                s.add_layer(Dialog::info(StyledString::styled(msg_save_res.err().unwrap(),
                    RED,
                )).title("Fatal error saving messages"));
            }
        })
        .button("Info", |s| {
            let content = "\nThe password you are setting up for dagchat is used to encrypt your accounts, messages (If 'Encrypt and save' messages setting is selected) and address book when they are saved on your device. It should be strong and contain a range of characters (UPPERCASE, lowercase, numb3rs and symbo!s). Without this password, dagchat will not be able to decrypt any of your saved accounts, messages or address book.";
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
            .button("Back", |s| show_accounts(s)),
    );
}

fn create_success(s: &mut Cursive, mnemonic: String, seed: String) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let mut content = StyledString::styled("\nMnemonic\n", data.coin.colour);
    content.append(StyledString::styled(&mnemonic, OFF_WHITE));
    content.append(StyledString::styled("\n\nSeed\n", data.coin.colour));
    content.append(StyledString::styled(&seed, OFF_WHITE));
    s.add_layer(
        Dialog::around(TextView::new(content).max_width(80))
            .h_align(HAlign::Center)
            .button("Load", |s| load_current_account(s))
            .button("Back", |s| show_accounts(s))
            .button("Copy mnemonic", move |s| copy_to_clip(s, mnemonic.clone()))
            .button("Copy seed", move |s| copy_to_clip(s, seed.clone()))
            .title("Successfully generated new account")
            .max_width(80),
    );
}
