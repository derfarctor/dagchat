use super::*;
use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{self, Config};

const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 12;

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

pub fn check_setup(s: &mut Cursive) {
    if let Some(mut data_dir) = dirs::data_dir() {
        data_dir = data_dir.join("dagchat");
        if data_dir.exists() {
            load_accounts(s, data_dir);
        } else {
            fs::create_dir(data_dir.clone()).unwrap_or_else(|e| {
                let content = format!(
                    "Failed to create a data folder for dagchat at path: {:?}\nError: {}",
                    data_dir, e
                );
                s.add_layer(Dialog::info(content))
            });
            if data_dir.exists() {
                load_accounts(s, data_dir);
            } else {
                return;
            }
        }
    } else {
        s.add_layer(Dialog::info(
            "Error locating the application data folder on your system.",
        ));
    }
}

fn load_accounts(s: &mut Cursive, data_path: PathBuf) {
    s.pop_layer();
    let accounts_file = data_path.join("accounts.dagchat");
    if accounts_file.exists() {
        let encrypted_bytes = fs::read(&accounts_file).unwrap_or_else(|e| {
            let content = format!(
                "Failed to read accounts.dagchat file at path: {:?}\nError: {}",
                accounts_file, e
            );
            s.add_layer(Dialog::info(content));
            vec![]
        });
        if encrypted_bytes.is_empty() {
            show_accounts(s);
            return;
        }
        let data = &mut s.user_data::<UserData>().unwrap();
        data.encrypted_accounts = encrypted_bytes;
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
    let bytes = decrypt_accounts(&data.encrypted_accounts, &password);
    if bytes.is_none() {
        s.add_layer(Dialog::info("Password was incorrect."));
        return;
    }
    data.password = password.to_string();
    
    let accounts_res = bincode::deserialize(&bytes.unwrap()[..]);
    if accounts_res.is_err() {
        show_accounts(s);
        s.add_layer(Dialog::info(StyledString::styled("Error parsing accounts.dagchat file. File was either corrupted or edited outside of dagchat.", Color::Dark(BaseColor::Red))));
    } else {
        data.accounts = accounts_res.unwrap();
        show_accounts(s);
    }
}

fn write_accounts(encrypted_bytes: Vec<u8>) -> bool {
    let mut success = false;
    if let Some(data_dir) = dirs::data_dir() {
        let accounts_file = data_dir.join("dagchat/accounts.dagchat");
        // Commented out code assumed unnecessary.
        //if !accounts_file.exists() {
        //    fs::File::create(&accounts_file).expect(&format!(
        //        "Unable to create accounts.dagchat at path: {:?}",
        //        &accounts_file
        //    ));
        //}
        //if accounts_file.exists() {
        success = true;
        fs::write(&accounts_file, encrypted_bytes).unwrap_or_else(|e| {
            success = false;
            eprintln!(
                "Failed to write to accounts.dagchat file at path: {:?}\nError: {}",
                accounts_file, e
            );
        });
        //}
    }
    success
}

fn save_accounts(s: &mut Cursive) -> bool {
    let data = &s.user_data::<UserData>().unwrap();
    let success: bool;
    if data.accounts.is_empty() {
        success = write_accounts(vec![]);
    } else {
        let encrypted_bytes = encrypt_accounts(&data.accounts, &data.password);
        success = write_accounts(encrypted_bytes);
    }
    success
}

fn derive_key(password: &str, salt: &[u8]) -> Vec<u8> {
    let config = Config::default();
    let hash = argon2::hash_raw(password.as_bytes(), salt, &config).unwrap();
    hash
}

fn encrypt_accounts(accounts: &Vec<Account>, password: &str) -> Vec<u8> {
    let mut csprng = rand::thread_rng();
    let mut salt = [0u8; SALT_LENGTH];
    csprng.fill_bytes(&mut salt);

    let key_bytes = derive_key(password, &salt);
    let key = GenericArray::from_slice(&key_bytes);
    let encoded: Vec<u8> = bincode::serialize(accounts).unwrap();
    let aead = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    csprng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = aead.encrypt(nonce, &encoded[..]).unwrap();
    let mut encrypted_bytes = Vec::with_capacity(12 + ciphertext.len());
    encrypted_bytes.extend(salt);
    encrypted_bytes.extend(nonce);
    encrypted_bytes.extend(ciphertext);
    encrypted_bytes
}

fn decrypt_accounts(encrypted_bytes: &[u8], password: &str) -> Option<Vec<u8>> {
    let salt = &encrypted_bytes[..SALT_LENGTH];

    let key_bytes = derive_key(password, salt);
    let key = GenericArray::from_slice(&key_bytes);

    let aead = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&encrypted_bytes[SALT_LENGTH..SALT_LENGTH + IV_LENGTH]);
    let encrypted = &encrypted_bytes[SALT_LENGTH + IV_LENGTH..];
    let decrypted = aead.decrypt(nonce, encrypted);
    if decrypted.is_err() {
        return None;
    }
    Some(decrypted.unwrap())
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
                    .child(Dialog::around(select).padding_lrtb(1, 1, 0, 0).title("Accounts"))
                    .child(DummyView)
                    .child(DummyView)
                    .child(buttons),
            )
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
            .button("Begin", |s| {     s.pop_layer();
                receive::load_receivables(s)})
            .button("Back", |s| {
                s.pop_layer();
                s.pop_layer();
                show_accounts(s)});
            if account.mnemonic.is_some() {
                let mnemonic = account.mnemonic.as_ref().unwrap().clone();
                origin = "loaded from mnemonic phrase";
                add_idx_button(&mut outer);
                outer.add_button("Copy mnemonic",move |s| {
                    copy_to_clip(s, mnemonic.clone());
                });
            } else if account.seed.is_some() {
                origin = "loaded from seed";
                let seed = hex::encode(account.seed.unwrap());
                add_idx_button(&mut outer);
                outer.add_button("Copy seed", move |s| {
                    copy_to_clip(s, seed.clone());
                });
            } else {
                origin = "loaded from private key";
                let key = hex::encode(account.private_key);
                outer.add_button("Copy private key", move |s| {
                    copy_to_clip(s, key.clone());
                });
                extra = String::from("");
            }
            outer.add_button("Remove", move |s| remove_account(s, focus));
            let content = LinearLayout::vertical()
            .child(DummyView)
            .child(TextView::new(StyledString::styled(format!("Account address{}",extra), OFF_WHITE)))
            .child(TextView::new(StyledString::styled(&account.address, data.coin.colour)))
            .child(DummyView)
            .child(TextView::new(StyledString::styled("Account type", OFF_WHITE)))
            .child(TextView::new(StyledString::styled(origin, data.coin.colour)));
            s.add_layer(outer.content(content)
            .title(format!("Account {}", focus+1)));
        }
    }
}

fn add_idx_button(dialog: &mut Dialog) {
    dialog.add_button("Index", move |s| {
        s.add_layer(Dialog::around(LinearLayout::vertical()
    .child(DummyView)
    .child(TextView::new("Account index (0 - 4,294,967,295)"))
    .child(EditView::new().on_submit(process_idx).with_name("index")))
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
    .title("Change account index"))
    });
}

fn process_idx(s: &mut Cursive, idx: &str) {
    let index_res: Result<u32, _> = idx.parse();
    if index_res.is_err() {
        s.add_layer(Dialog::info("Error: index was not an integer within the valid range."));
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
        Color::Light(BaseColor::Red),
    );
    s.add_layer(Dialog::around(LinearLayout::vertical().child(DummyView).child(TextView::new(warning)).child(DummyView))
    .h_align(HAlign::Center)
    .button("Back", |s| {
        s.pop_layer();
    })    
    .button("Confirm", move |s| {
            let data = &mut s.user_data::<UserData>().unwrap();
            data.accounts.remove(idx);
            let success = save_accounts(s);
            s.pop_layer();
            s.pop_layer();
            s.pop_layer();
            show_accounts(s);
            if !success {
                s.add_layer(Dialog::info(StyledString::styled("Error saving accounts data. Account may not be removed upon relaunching dagchat.", Color::Light(BaseColor::Red))));
            }
        })
        .title("Confirm account deletion")
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
    let account = Account::from_seed(Some(seed_bytes), &data.coin.prefix);
    setup_account(s, account, move |s| new_success(s, hex::encode(seed_bytes)));
}

fn setup_account<F: 'static>(s: &mut Cursive, account: Account, on_success: F)
where
    F: Fn(&mut Cursive),
{
    let warning = StyledString::styled(
        "Without this password, dagchat can not restore your accounts since they are saved in an encrypted format. Always backup or write down your mnemonics, seeds or keys elsewhere in case you forget your password.", Color::Light(BaseColor::Red));

    let data = &mut s.user_data::<UserData>().unwrap();
    data.accounts.push(account);
    data.acc_idx = data.accounts.len();
    // First account added, setup password
    // Need to add password confirmation here
    if data.accounts.len() == 1 {
        s.add_layer(
            Dialog::around(
                LinearLayout::vertical()
                    .child(DummyView)
                    .child(EditView::new().secret().with_name("password"))
                    .child(DummyView)
                    .child(TextView::new(warning)),
            )
            .h_align(HAlign::Center)
            .button("Submit", move |s| {
                let password = s
                    .call_on_name("password", |view: &mut EditView| view.get_content())
                    .unwrap();
                let data = &mut s.user_data::<UserData>().unwrap();
                data.password = password.to_string();
                let success = save_accounts(s);
                s.pop_layer();
                if success {
                    on_success(s);
                } else {
                    show_accounts(s);
                    s.add_layer(Dialog::info(StyledString::styled("Error saving accounts data. Account may not remain upon relaunching dagchat.", Color::Light(BaseColor::Red))));
                }
            })
            .title("Create a password for dagchat")
            .max_width(80),
        );
    } else {
        let success = save_accounts(s);
        if success {
            on_success(s);
        } else {
            show_accounts(s);
            s.add_layer(Dialog::info(StyledString::styled("Error saving accounts data. Account may not remain upon relaunching dagchat.", Color::Light(BaseColor::Red))));
        }
    }
}

fn import_success(s: &mut Cursive, content: &str) {
    s.add_layer(
        Dialog::around(TextView::new(content).max_width(80))
            .button("Begin", |s| receive::load_receivables(s))
            .button("Back", |s| show_accounts(s)),
    );
}

fn new_success(s: &mut Cursive, seed: String) {
    let data = &mut s.user_data::<UserData>().unwrap();
    let mut content = StyledString::styled("Successfully generated new account with seed: ", data.coin.colour);
    content.append(StyledString::styled(&seed, OFF_WHITE));
    s.add_layer(
        Dialog::around(TextView::new(content).max_width(80))
            .button("Copy seed", move |s| copy_to_clip(s, seed.clone()))
            .button("Begin", |s| receive::load_receivables(s))
            .button("Back", |s| show_accounts(s)),
    );
}
