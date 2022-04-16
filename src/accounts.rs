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
    pub key_idx: u128,
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
        let private_key = dcutil::get_private_key(seed);
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
                accounts_file.display(),
                e
            );
            s.add_layer(Dialog::info(content));
            vec![]
        });
        if encrypted_bytes.is_empty() {
            show_accounts(s);
            return;
        }
        s.add_layer(Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Enter password:"))
                .child(TextArea::new().with_name("password").max_width(80))
                .child(Button::new("Submit", move |s| {
                    let mut password = String::from("");
                    s.call_on_name("password", |view: &mut TextArea| {
                        password = String::from(view.get_content());
                    });

                    let bytes = decrypt_accounts(&encrypted_bytes, &password);
                    if bytes.is_none() {
                        s.add_layer(Dialog::info("Incorrect password."));
                        return;
                    }
                    let data = &mut s.user_data::<UserData>().unwrap();
                    data.password = password;
                    data.accounts = bincode::deserialize(&bytes.unwrap()[..]).unwrap();
                    show_accounts(s);
                })),
        ));
    } else {
        show_accounts(s);
    }
}

fn write_accounts(encrypted_bytes: Vec<u8>) -> bool {
    let mut success = false;
    if let Some(data_dir) = dirs::data_dir() {
        let accounts_file = data_dir.join("dagchat/accounts.dagchat");
        if !accounts_file.exists() {
            fs::File::create(&accounts_file).expect(&format!(
                "Unable to create accounts.dagchat at path: {:?}",
                &accounts_file
            ));
        }
        if accounts_file.exists() {
            success = true;
            fs::write(&accounts_file, encrypted_bytes).unwrap_or_else(|e| {
                success = false;
                eprintln!(
                    "Failed to write to accounts.dagchat file at path: {:?}\nError: {}",
                    accounts_file, e
                );
            });
        }
    }
    success
}

fn save_accounts(s: &mut Cursive) {
    let data = &s.user_data::<UserData>().unwrap();
    let success: bool;
    if data.accounts.is_empty() {
        success = write_accounts(vec![]);
    } else {
        let encrypted_bytes = encrypt_accounts(&data.accounts, &data.password);
        success = write_accounts(encrypted_bytes);
    }
    if !success {
        s.add_layer(Dialog::info("Error saving accounts data."));
    }
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
    //Scroll view on left, iter on accounts and show them all
    //Have 'Guest' button
    //Have add, remove account, change password buttons on right
    //When importing seed/mnemonic, option to specify index
    let buttons = LinearLayout::vertical()
        .child(Button::new("Add account", |s| add_account(s)))
        .child(Button::new("Remove account", |s| remove_account(s)));

    let select = SelectView::<String>::new()
        .on_submit(select_account)
        .with_name("accounts")
        .scrollable()
        .max_height(10);

    s.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(select)
                .child(DummyView)
                .child(buttons),
        )
        .title("Select account"),
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

fn select_account(s: &mut Cursive, name: &str) {
    let select = s.find_name::<SelectView<String>>("accounts").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No account selected.")),
        Some(focus) => {
            // If from mnemonic or seed, let user choose id. Else start.
            let data = &mut s.user_data::<UserData>().unwrap();
            data.acc_idx = focus;
            receive::load_receivables(s);
        }
    }
}

fn remove_account(s: &mut Cursive) {
    let select = s.find_name::<SelectView<String>>("accounts").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No account selected.")),
        Some(focus) => {
            // Add backup button to copy mnemonic/seed to clip
            s.add_layer(Dialog::around(TextView::new("Confirm account deletion. If you have not backed up this account, it will be lost forever.").max_width(80))
                .button("Confirm", move |s| {
                    let data = &mut s.user_data::<UserData>().unwrap();
                    data.accounts.remove(focus);
                    save_accounts(s);
                    s.pop_layer();
                    show_accounts(s);
                })
                .button("Back", |s| {
                    s.pop_layer();
                })
            );
        }
    }
}

fn add_account(s: &mut Cursive) {
    s.pop_layer();
    let data = &s.user_data::<UserData>().unwrap();
    let coin = &data.coin.name;
    let colour = data.coin.colour;
    let content = format!("Choose a way to import your {} wallet.", coin);
    s.add_layer(
        Dialog::text(StyledString::styled(content, colour))
            .title("Import account")
            .h_align(HAlign::Center)
            .button("Mnemonic", |s| from_mnemonic(s))
            .button("Seed", |s| from_seed_or_key(s, String::from("seed")))
            .button("New", |s| new_account(s)),
    );
}

fn from_mnemonic(s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .title("Enter your 24 word mnemonic")
            .padding_lrtb(1, 1, 1, 0)
            .content(EditView::new().with_name("mnemonic").fixed_width(29))
            .h_align(HAlign::Center)
            .button("Done", |s| {
                let mnemonic = s
                    .call_on_name("mnemonic", |view: &mut EditView| view.get_content())
                    .unwrap();
                let seed = validate_mnemonic(&mnemonic);
                let content;
                s.pop_layer();
                if !mnemonic.is_empty() && seed.is_some() {
                    let seed_bytes = seed.unwrap();
                    let data = &s.user_data::<UserData>().unwrap();
                    let account = Account::from_mnemonic(
                        Some(String::from(&*mnemonic)),
                        Some(seed_bytes),
                        &data.coin.prefix,
                    );
                    setup_account(s, account, |s| {
                        import_success(s, "Successfully imported account from mnemonic phrase.")
                    });
                } else {
                    content = "The mnemonic you entered was not valid.";
                    s.add_layer(
                        Dialog::around(TextView::new(content)).button("Back", |s| from_mnemonic(s)),
                    );
                    return;
                }
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

fn from_seed_or_key(s: &mut Cursive, seed_or_key: String) {
    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .title(format!("Enter your {}", seed_or_key))
            .padding_lrtb(1, 1, 1, 0)
            .content(EditView::new().with_name("seedorkey").fixed_width(29))
            .h_align(HAlign::Center)
            .button("Done", move |s| {
                let raw_seedorkey = s
                    .call_on_name("seedorkey", |view: &mut EditView| view.get_content())
                    .unwrap();
                let seedorkey = raw_seedorkey.trim();
                if seedorkey.len() != 64 {
                    s.add_layer(Dialog::info(format!(
                        "Error: {} was invalid - not 64 characters long.",
                        seed_or_key
                    )));
                    return;
                }
                let bytes_opt = hex::decode(seedorkey);
                if bytes_opt.is_err() {
                    s.add_layer(Dialog::info(format!(
                        "Error: {} was invalid - failed to decode hex.",
                        seed_or_key
                    )));
                    return;
                }
                let bytes = bytes_opt.unwrap();
                let seedorkey_bytes: [u8; 32] = bytes.try_into().unwrap();
                let data = &s.user_data::<UserData>().unwrap();
                let account: Account;
                if seed_or_key == "seed" {
                    account = Account::from_seed(Some(seedorkey_bytes), &data.coin.prefix);
                } else {
                    account = Account::from_private_key(seedorkey_bytes, &data.coin.prefix);
                }
                let content = format!("Successfully imported account from {}.", seed_or_key);
                setup_account(s, account, move |s| import_success(s, &content));
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
    let data = &mut s.user_data::<UserData>().unwrap();
    data.accounts.push(account);

    // First account added, setup password
    if data.accounts.len() == 1 {
        s.add_layer(Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Create a password. If you forget this, you will not be able to restore your accounts so make sure you have backed up the mnemonic phrases, seeds or private keys of your accounts.").max_width(80))
                .child(TextArea::new().with_name("password").max_width(80))
                .child(Button::new("Submit", move |s| {
                    let mut password = String::from("");
                    s.call_on_name("password", |view: &mut TextArea| {
                        password = String::from(view.get_content());
                    });
                    let data = &mut s.user_data::<UserData>().unwrap();
                    data.password = password;
                    save_accounts(s);
                    s.pop_layer();
                    on_success(s);
                }))));
    } else {
        save_accounts(s);
        on_success(s);
    }
}

fn import_success(s: &mut Cursive, content: &str) {
    s.add_layer(
        Dialog::around(TextView::new(content).max_width(80))
            .button("Begin", |s| alpha_info(s))
            .button("Back", |s| show_accounts(s)),
    );
}

fn new_success(s: &mut Cursive, seed: String) {
    let mut content = StyledString::plain("Successfully generated new account with seed: ");
    content.append(StyledString::styled(&seed, OFF_WHITE));
    s.add_layer(
        Dialog::around(TextView::new(content).max_width(80))
            .button("Copy seed", move |s| copy_to_clip(s, seed.clone()))
            .button("Begin", |s| alpha_info(s))
            .button("Back", |s| show_accounts(s)),
    );
}
