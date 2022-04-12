use bigdecimal::BigDecimal;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use data_encoding::Encoding;
use data_encoding_macro::new_encoding;
use ecies_ed25519;
use ed25519_dalek;
use serde;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str;
use std::str::FromStr;
// Will do something dynamic with reps in future
use crate::defaults;

// Used to update progress bar in cursive app
// Each counter has 1000 ticks
use cursive::utils::Counter;

const ADDR_ENCODING: Encoding = new_encoding! {
    symbols: "13456789abcdefghijkmnopqrstuwxyz",
    check_trailing_bits: false,
};

const MULTI: &str = "100000000000000000000000000000";

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInfoResponse {
    frontier: String,
    open_block: String,
    representative_block: String,
    balance: String,
    modified_timestamp: String,
    block_count: String,
    account_version: String,
    confirmation_height: String,
    confirmation_height_frontier: String,
    representative: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default)]
    pub account: String,
    previous: String,
    representative: String,
    balance: String,
    link: String,
    signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceivableRequest {
    action: String,
    account: String,
    source: bool,
    sorting: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceivableResponse {
    blocks: ReceivableBlocks,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceivableBlocks {
    #[serde(flatten)]
    data: HashMap<String, ReceivableBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReceivableBlock {
    amount: String,
    source: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receivable {
    pub hash: String,
    pub message: Option<Message>,
    pub amount: u128,
    // Used for seeing message sender in app
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub head: Option<BlockResponse>,
    pub root_hash: String,
    pub blocks: u64,
    pub plaintext: String,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            head: None,
            root_hash: String::from(""),
            blocks: 0,
            plaintext: String::from(""),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcessRequest {
    action: String,
    json_block: String,
    do_work: bool,
    subtype: String,
    block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProcessResponse {
    hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockRequest {
    action: String,
    json_block: bool,
    hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockResponse {
    block_account: String,
    amount: String,
    balance: String,
    height: String,
    local_timestamp: String,
    successor: String,
    confirmed: String,
    pub contents: Block,
    subtype: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HistoryRequest {
    action: String,
    account: String,
    count: u64,
    head: String,
    reverse: bool,
    raw: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryResponse {
    account: String,
    history: Vec<Block>,
    #[serde(default)]
    next: String,
}

pub fn send_message(
    private_key_bytes: &[u8; 32],
    target_address: String,
    raw: u128,
    message: String,
    node_url: &str,
    addr_prefix: &str,
    counter: &Counter,
) {
    let public_key_bytes = to_public_key(&target_address);
    let mut message = message.clone();
    let pad = (message.len() + 28) % 32;
    for _ in 0..(32 - pad) {
        message.push_str(" ");
    }
    let public_key = ecies_ed25519::PublicKey::from_bytes(&public_key_bytes).unwrap();
    counter.tick(50);
    //println!("Encrypting message for send: {}", message);
    let mut csprng = rand::thread_rng();
    let encrypted_bytes =
        ecies_ed25519::encrypt(&public_key, message.as_bytes(), &mut csprng).unwrap();
    let blocks_needed = ((60 + message.len()) / 32) + 1;
    counter.tick(50);
    //println!("Blocks needed: {}", blocks_needed);

    let mut block_data = [0u8; 32];
    let mut first_block_hash = [0u8; 32];

    // Derive sender's address
    let sender_pub = ed25519_dalek::PublicKey::from(
        &ed25519_dalek::SecretKey::from_bytes(private_key_bytes).unwrap(),
    );
    let sender_address = get_address(sender_pub.as_bytes(), addr_prefix);

    // Set up the previous block hash and balance to start publishing blocks
    // Also note the representative from before sending, in order to change back afterwards
    let account_info = get_account_info(&sender_address, node_url).unwrap();
    let mut last_block_hash = get_32_bytes(&account_info.frontier);
    let mut balance = get_balance(&account_info);
    let representative = to_public_key(&account_info.representative);

    let mut link = [0u8; 32];
    let mut sub = String::from("change");
    counter.tick(100);
    let x = 800usize / blocks_needed;
    for block_num in 0..blocks_needed {
        counter.tick(x);
        let start = 32 * block_num;
        let end = 32 * (block_num + 1);
        if block_num == blocks_needed - 1 {
            // Last block sent is the send block with 1 raw to recipient
            // Link is the recipient
            // Rep is the hash of the first block in the message
            block_data = first_block_hash;
            balance -= raw;
            link = public_key_bytes;
            sub = String::from("send");
        } else {
            block_data.copy_from_slice(&encrypted_bytes[start..end]);
        }
        //println!("Block data as addr: {:?}", get_address(&block_data, addr_prefix));
        let block_hash = get_block_hash(
            private_key_bytes,
            &block_data,
            &last_block_hash,
            &link,
            balance,
        );
        let block = get_signed_block(
            private_key_bytes,
            &block_data,
            &last_block_hash,
            &link,
            balance,
            &block_hash,
            addr_prefix,
        );
        if block_num == 0 {
            first_block_hash = block_hash;
        }
        last_block_hash = block_hash;
        publish_block(block, sub.clone(), node_url);
    }
    // Change representative to what it was at the start
    link = [0u8; 32];
    sub = String::from("change");
    let block_hash = get_block_hash(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        balance,
    );
    let block = get_signed_block(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        balance,
        &block_hash,
        &addr_prefix,
    );
    publish_block(block, sub.clone(), node_url);
}

pub fn change_rep(
    private_key_bytes: &[u8; 32],
    account_info: AccountInfoResponse,
    rep_address: &str,
    node_url: &str,
    addr_prefix: &str,
) {
    let last_block_hash = get_32_bytes(&account_info.frontier);
    let balance = get_balance(&account_info);
    let representative = to_public_key(rep_address);
    let link = [0u8; 32];
    let block_hash = get_block_hash(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        balance,
    );
    let block = get_signed_block(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        balance,
        &block_hash,
        addr_prefix,
    );
    publish_block(block, String::from("change"), node_url);
}
pub fn receive_block(
    private_key_bytes: &[u8; 32],
    send_block: &str,
    amount: u128,
    address: &str,
    node_url: &str,
    addr_prefix: &str,
    counter: &Counter,
) {
    let account_info_opt = get_account_info(address, node_url);
    counter.tick(300);
    let mut last_block_hash = [0u8; 32];
    let mut new_balance = amount;
    let representative: [u8; 32];
    let link = get_32_bytes(send_block);
    if account_info_opt.is_none() {
        // OPEN BLOCK
        if addr_prefix == "nano_" {
            representative = to_public_key(defaults::DEFAULT_REP_NANO);
        } else if addr_prefix == "ban_" {
            representative = to_public_key(defaults::DEFAULT_REP_BANANO);
        } else {
            panic!("Unknown network... no default rep to open account.");
        }
    } else {
        let account_info = account_info_opt.unwrap();
        last_block_hash = get_32_bytes(&account_info.frontier);
        let balance = get_balance(&account_info);
        new_balance = balance + amount;
        representative = to_public_key(&account_info.representative);
    }
    counter.tick(200);
    let block_hash = get_block_hash(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        new_balance,
    );
    let signed_block = get_signed_block(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        new_balance,
        &block_hash,
        addr_prefix,
    );
    counter.tick(200);
    publish_block(signed_block, String::from("receive"), node_url);
    counter.tick(200);
}

pub fn send(
    private_key_bytes: &[u8; 32],
    address: String,
    raw: u128,
    node_url: &str,
    addr_prefix: &str,
    counter: &Counter,
) {
    // Derive sender's address
    let sender_pub = ed25519_dalek::PublicKey::from(
        &ed25519_dalek::SecretKey::from_bytes(private_key_bytes).unwrap(),
    );
    let sender_address = get_address(sender_pub.as_bytes(), addr_prefix);

    // Safe because account must be opened to have got this far
    let account_info = get_account_info(&sender_address, node_url).unwrap();
    counter.tick(400);
    let last_block_hash = get_32_bytes(&account_info.frontier);
    let new_balance = get_balance(&account_info) - raw;
    let representative = to_public_key(&account_info.representative);
    let link = to_public_key(&address);
    counter.tick(100);
    let block_hash = get_block_hash(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        new_balance,
    );
    let signed_block = get_signed_block(
        private_key_bytes,
        &representative,
        &last_block_hash,
        &link,
        new_balance,
        &block_hash,
        addr_prefix,
    );
    counter.tick(100);
    publish_block(signed_block, String::from("send"), node_url);
    counter.tick(400);
}
pub fn has_message(head_hash: &str, node_url: &str) -> Option<Message> {
    // Assured to exist before being passed to function
    let head_block = get_block_info(head_hash, node_url).unwrap();
    let representative_bytes = to_public_key(&head_block.contents.representative);
    let root_hash = hex::encode(representative_bytes);
    let root_opt = get_block_info(&root_hash, node_url);
    // Check if block exists. If it does, then it points toward a message root
    if let Some(root_block) = root_opt {
        let head_height: u64 = head_block.height.parse().unwrap();
        let root_height: u64 = root_block.height.parse().unwrap();
        let message_block_count = head_height - root_height;
        let message = Message {
            blocks: message_block_count,
            head: Some(head_block),
            root_hash: root_hash,
            plaintext: String::from(""),
        };
        return Some(message);
    } else {
        return None;
    }
}

pub fn read_message(
    private_key_bytes: &[u8; 32],
    target_address: &str,
    root_hash: &str,
    blocks: u64,
    node_url: &str,
) -> String {
    let message_blocks = get_history(target_address, root_hash, blocks, node_url);

    let encrypted_bytes = extract_message(message_blocks);

    let dalek = ed25519_dalek::SecretKey::from_bytes(private_key_bytes).unwrap();
    let expanded_bytes = ed25519_dalek::ExpandedSecretKey::from(&dalek);
    let private_key =
        ecies_ed25519::SecretKey::from_bytes(&expanded_bytes.to_bytes()[0..32]).unwrap();
    let decrypted = ecies_ed25519::decrypt(&private_key, &encrypted_bytes);
    if decrypted.is_err() {
        return String::from("Error decrypting message.");
    }
    let plaintext = String::from_utf8(decrypted.unwrap()).unwrap();
    //println!("{}", plaintext);
    plaintext
}

pub fn find_incoming(target_address: &str, node_url: &str) -> Vec<Receivable> {
    let request = ReceivableRequest {
        action: String::from("pending"),
        account: String::from(target_address),
        source: true,
        /* Maybe there's an efficient way to use this working backwards. Not implemented yet. */
        sorting: true,
    };

    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url);

    // Response from node if there are no receivable blocks
    //println!("{}", response);
    if response.contains("\"blocks\": \"\"") {
        return vec![];
    }
    let receivable: ReceivableResponse = serde_json::from_str(&response).unwrap();

    let blocks = receivable.blocks.data;
    let mut incoming: Vec<Receivable> = vec![];
    for block in blocks {
        //println!("{} info: {:?}", block.0, block.1);
        incoming.push(Receivable {
            hash: block.0,
            amount: block.1.amount.parse().unwrap(),
            source: block.1.source,
            message: Default::default(),
        });
    }
    incoming
}

pub fn get_block_info(hash: &str, node_url: &str) -> Option<BlockResponse> {
    let request = BlockRequest {
        action: String::from("block_info"),
        json_block: true,
        hash: String::from(hash),
    };
    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url);
    //println!("{}", response);
    if response.contains("Block not found") {
        return None;
    }
    let block_response: BlockResponse = serde_json::from_str(&response).unwrap();
    Some(block_response)
}

pub fn get_account_info(address: &str, node_url: &str) -> Option<AccountInfoResponse> {
    // Change this to AccountInfoRequest struct
    let body_json = json!({
        "action": "account_info",
        "account": String::from(address),
        "representative": true
    });

    let body = body_json.to_string();
    let resp_string = post_node(body, node_url);
    if resp_string.contains("Account not found") {
        return None;
    }
    let account_info: AccountInfoResponse = serde_json::from_str(&resp_string).unwrap();
    Some(account_info)
}

pub fn get_32_bytes(string: &str) -> [u8; 32] {
    let bytes = hex::decode(string).unwrap();
    let mut array = [0u8; 32];
    array.copy_from_slice(bytes.as_slice());
    array
}

pub fn get_balance(info: &AccountInfoResponse) -> u128 {
    let balance: u128 = info.balance.parse().unwrap();
    balance
}

pub fn get_history(target_address: &str, head: &str, length: u64, node_url: &str) -> Vec<Block> {
    let request = HistoryRequest {
        action: String::from("account_history"),
        account: String::from(target_address),
        count: length,
        head: String::from(head),
        reverse: true,
        raw: true,
    };
    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url);
    //println!("{}", response);
    let history_info: HistoryResponse = serde_json::from_str(&response).unwrap();
    history_info.history
}

pub fn extract_message(blocks: Vec<Block>) -> Vec<u8> {
    let mut encrypted_bytes = vec![];
    for block in blocks {
        let block_data = to_public_key(&block.representative);
        encrypted_bytes.extend(&block_data);
    }
    encrypted_bytes
}

// Could do with rework
pub fn post_node(body: String, node_url: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(node_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .unwrap();

    if res.status().is_success() {
        //eprintln!("Successfully communicated with node");
        let response_str = res.text().unwrap();
        return response_str;
    } else {
        //eprintln!("Issue. Status: {}", res.status());
    }
    String::from("Failed")
}

pub fn publish_block(block: Block, sub: String, node_url: &str) -> String {
    let request = ProcessRequest {
        action: String::from("process"),
        json_block: String::from("true"),
        do_work: true,
        subtype: sub,
        block: block,
    };

    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url);
    response
}

pub fn get_block_hash(
    priv_k: &[u8; 32],
    rep: &[u8; 32],
    previous: &[u8; 32],
    link: &[u8; 32],
    balance: u128,
) -> [u8; 32] {
    let secret = ed25519_dalek::SecretKey::from_bytes(priv_k).unwrap();
    let public = ed25519_dalek::PublicKey::from(&secret);

    let mut hasher = Blake2bVar::new(32).unwrap();
    let mut buf = [0u8; 32];

    hasher.update(
        &hex::decode("0000000000000000000000000000000000000000000000000000000000000006").unwrap(),
    );
    hasher.update(public.as_bytes());
    hasher.update(previous);
    hasher.update(rep);

    let mut x = format!("{:x}", &balance);
    //println!("Non padded: {}", x);
    while x.len() < 32 {
        x = format!("0{}", x);
    }
    //println!("{}", x);
    //println!("{}", x.len());
    //println!("{:?}", hex::decode(&x).unwrap());
    hasher.update(&hex::decode(x).unwrap());
    hasher.update(link);
    hasher.finalize_variable(&mut buf).unwrap();
    buf
}

pub fn get_signed_block(
    priv_k: &[u8; 32],
    rep: &[u8; 32],
    previous: &[u8; 32],
    link: &[u8; 32],
    balance: u128,
    block_hash: &[u8; 32],
    addr_prefix: &str,
) -> Block {
    let secret = ed25519_dalek::SecretKey::from_bytes(priv_k).unwrap();
    let public = ed25519_dalek::PublicKey::from(&secret);
    let expanded_secret = ed25519_dalek::ExpandedSecretKey::from(&secret);

    let internal_signed =
        expanded_secret.sign(block_hash, &ed25519_dalek::PublicKey::from(&secret));

    //println!("{:?}", internal_signed);
    let y = internal_signed.to_bytes();
    let z = hex::encode(&y);
    //println!("{}", z);

    //let work = generate_work(&previous, "banano");
    let block = Block {
        type_name: String::from("state"),
        account: get_address(public.as_bytes(), addr_prefix),
        previous: hex::encode(previous),
        representative: get_address(rep, addr_prefix),
        balance: balance.to_string(),
        link: hex::encode(link),
        signature: z,
    };

    block
}

pub fn validate_mnemonic(mnemonic: &str) -> Option<[u8; 32]> {
    let (mnemonic, valid) = get_num_equivalent(mnemonic);
    if !valid {
        return None;
    }
    let mut bits = [false; 24 * 11];
    for i in 0..24 {
        for j in 0..11 {
            bits[i * 11 + j] = mnemonic[i] >> (10 - j) & 1 == 1;
        }
    }

    let mut entropy = [0u8; 32];
    for i in 0..32 {
        for j in 0..8 {
            if bits[i * 8 + j] {
                entropy[i] += 1 << (7 - j);
            }
        }
    }

    let mut hasher = Sha256::new();
    Digest::update(&mut hasher, &entropy);
    let check = hasher.finalize();
    for i in 0..8 {
        if bits[8 * 32 + i] != ((check[i / 8] & (1 << (7 - (i % 8)))) > 0) {
            return None;
        }
    }

    Some(entropy)
}

pub fn get_private_key(seed_bytes: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Blake2bVar::new(32).unwrap();
    let mut buf = [0u8; 32];
    hasher.update(seed_bytes);
    hasher.update(&[0u8; 4]);
    hasher.finalize_variable(&mut buf).unwrap();
    buf
}

pub fn get_address(pub_key_bytes: &[u8], prefix: &str) -> String {
    let mut pub_key_vec = pub_key_bytes.to_vec();
    let mut h = [0u8; 3].to_vec();
    h.append(&mut pub_key_vec);
    let checksum = ADDR_ENCODING.encode(&compute_address_checksum(pub_key_bytes));
    let address = {
        let encoded_addr = ADDR_ENCODING.encode(&h);
        let mut addr = String::from(prefix);
        addr.push_str(encoded_addr.get(4..).unwrap());
        addr.push_str(&checksum);
        addr
    };
    address
}

pub fn to_public_key(addr: &str) -> [u8; 32] {
    let mut encoded_addr: String;
    let parts: Vec<&str> = addr.split("_").collect();
    encoded_addr = String::from(parts[1].get(0..52).unwrap());
    encoded_addr.insert_str(0, "1111");
    let mut pub_key_vec = ADDR_ENCODING.decode(encoded_addr.as_bytes()).unwrap();
    pub_key_vec.drain(0..3);
    pub_key_vec.as_slice().try_into().unwrap()
}

// Could be to_public_key and have return value tuple with bool
// like get_num_equivalent but this is faster since to_public_key
// is used on confirmed valid addresses frequently in lib
pub fn validate_address(addr: &str) -> bool {
    let mut encoded_addr: String;

    if !addr.contains("_") {
        return false;
    };
    let parts: Vec<&str> = addr.split("_").collect();

    // Minimum viable public representation
    if parts[1].len() < 52 {
        return false;
    };
    let checksum = String::from(parts[1].get(52..).unwrap());
    encoded_addr = String::from(parts[1].get(0..52).unwrap());
    encoded_addr.insert_str(0, "1111");

    let pub_key_vec = ADDR_ENCODING.decode(encoded_addr.as_bytes());

    // Lazily catch decoding error - return false
    let mut pub_key_vec = match pub_key_vec {
        Ok(pub_key_vec) => pub_key_vec,
        Err(_) => return false,
    };
    pub_key_vec.drain(0..3);
    let public_key_bytes: [u8; 32] = pub_key_vec.as_slice().try_into().unwrap();
    let real_checksum_bytes = compute_address_checksum(&public_key_bytes);
    let real_checksum = ADDR_ENCODING.encode(&real_checksum_bytes);

    //println!("Told: {} Real: {}", checksum, real_checksum);
    if checksum == real_checksum {
        return true;
    } else {
        return false;
    }
}

fn compute_address_checksum(pub_key_bytes: &[u8]) -> [u8; 5] {
    let mut hasher = Blake2bVar::new(5).unwrap();
    let mut buf = [0u8; 5];
    hasher.update(pub_key_bytes);
    hasher.finalize_variable(&mut buf).unwrap();
    buf.reverse();
    buf
}

pub fn wordlist_position(word: &str) -> u16 {
    let index = WORD_LIST.iter().position(|&w| w == word).unwrap();
    index as u16
}

pub fn get_num_equivalent(mnemonic: &str) -> ([u16; 24], bool) {
    let mut num_mnemonic: [u16; 24] = [0u16; 24];
    let words: Vec<&str> = mnemonic.split(" ").collect();
    for i in 0..24 {
        if WORD_LIST.contains(&words[i]) {
            num_mnemonic[i] = wordlist_position(&words[i]);
        } else {
            return (num_mnemonic, false);
        }
    }
    (num_mnemonic, true)
}

// Also validates if the amount submitted was possible
pub fn whole_to_raw(whole: String) -> Option<u128> {
    let amount = BigDecimal::from_str(&whole.trim());
    if amount.is_err() {
        return None;
    }
    let multi = BigDecimal::from_str(MULTI).unwrap();
    let amount_raw = amount.unwrap() * multi;
    if amount_raw.is_integer() {
        let raw_string = amount_raw.with_scale(0).to_string();
        let raw: u128 = raw_string.parse().unwrap();
        if raw == 0 {
            return None;
        } else {
            return Some(raw);
        }
    } else {
        return None;
    }
}

pub fn display_to_dp(raw: u128, dp: usize, ticker: &str) -> String {
    if raw < 100000 {
        return format!("{} raw", raw);
    } else {
        let raw_string = raw.to_string();
        let raw = BigDecimal::from_str(&raw_string).unwrap();
        let multi = BigDecimal::from_str(MULTI).unwrap();
        let adjusted = raw / multi;
        let mut s = adjusted.to_string();

        // If decimal part, trim to dp
        if s.contains(".") {
            let mut parts: Vec<&str> = s.split(".").collect();
            let real_dp = parts[1].len();
            if real_dp > dp {
                parts[1] = parts[1].get(0..dp).unwrap();
            }
            s = format!("{}.{}{}", parts[0], parts[1], ticker);
        } else {
            s = format!("{}{}", s, ticker);
        }

        return s;
    }
}

pub static WORD_LIST: [&str; 2048] = [
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd",
    "abuse", "access", "accident", "account", "accuse", "achieve", "acid", "acoustic", "acquire",
    "across", "act", "action", "actor", "actress", "actual", "adapt", "add", "addict", "address",
    "adjust", "admit", "adult", "advance", "advice", "aerobic", "affair", "afford", "afraid",
    "again", "age", "agent", "agree", "ahead", "aim", "air", "airport", "aisle", "alarm", "album",
    "alcohol", "alert", "alien", "all", "alley", "allow", "almost", "alone", "alpha", "already",
    "also", "alter", "always", "amateur", "amazing", "among", "amount", "amused", "analyst",
    "anchor", "ancient", "anger", "angle", "angry", "animal", "ankle", "announce", "annual",
    "another", "answer", "antenna", "antique", "anxiety", "any", "apart", "apology", "appear",
    "apple", "approve", "april", "arch", "arctic", "area", "arena", "argue", "arm", "armed",
    "armor", "army", "around", "arrange", "arrest", "arrive", "arrow", "art", "artefact", "artist",
    "artwork", "ask", "aspect", "assault", "asset", "assist", "assume", "asthma", "athlete",
    "atom", "attack", "attend", "attitude", "attract", "auction", "audit", "august", "aunt",
    "author", "auto", "autumn", "average", "avocado", "avoid", "awake", "aware", "away", "awesome",
    "awful", "awkward", "axis", "baby", "bachelor", "bacon", "badge", "bag", "balance", "balcony",
    "ball", "bamboo", "banana", "banner", "bar", "barely", "bargain", "barrel", "base", "basic",
    "basket", "battle", "beach", "bean", "beauty", "because", "become", "beef", "before", "begin",
    "behave", "behind", "believe", "below", "belt", "bench", "benefit", "best", "betray", "better",
    "between", "beyond", "bicycle", "bid", "bike", "bind", "biology", "bird", "birth", "bitter",
    "black", "blade", "blame", "blanket", "blast", "bleak", "bless", "blind", "blood", "blossom",
    "blouse", "blue", "blur", "blush", "board", "boat", "body", "boil", "bomb", "bone", "bonus",
    "book", "boost", "border", "boring", "borrow", "boss", "bottom", "bounce", "box", "boy",
    "bracket", "brain", "brand", "brass", "brave", "bread", "breeze", "brick", "bridge", "brief",
    "bright", "bring", "brisk", "broccoli", "broken", "bronze", "broom", "brother", "brown",
    "brush", "bubble", "buddy", "budget", "buffalo", "build", "bulb", "bulk", "bullet", "bundle",
    "bunker", "burden", "burger", "burst", "bus", "business", "busy", "butter", "buyer", "buzz",
    "cabbage", "cabin", "cable", "cactus", "cage", "cake", "call", "calm", "camera", "camp", "can",
    "canal", "cancel", "candy", "cannon", "canoe", "canvas", "canyon", "capable", "capital",
    "captain", "car", "carbon", "card", "cargo", "carpet", "carry", "cart", "case", "cash",
    "casino", "castle", "casual", "cat", "catalog", "catch", "category", "cattle", "caught",
    "cause", "caution", "cave", "ceiling", "celery", "cement", "census", "century", "cereal",
    "certain", "chair", "chalk", "champion", "change", "chaos", "chapter", "charge", "chase",
    "chat", "cheap", "check", "cheese", "chef", "cherry", "chest", "chicken", "chief", "child",
    "chimney", "choice", "choose", "chronic", "chuckle", "chunk", "churn", "cigar", "cinnamon",
    "circle", "citizen", "city", "civil", "claim", "clap", "clarify", "claw", "clay", "clean",
    "clerk", "clever", "click", "client", "cliff", "climb", "clinic", "clip", "clock", "clog",
    "close", "cloth", "cloud", "clown", "club", "clump", "cluster", "clutch", "coach", "coast",
    "coconut", "code", "coffee", "coil", "coin", "collect", "color", "column", "combine", "come",
    "comfort", "comic", "common", "company", "concert", "conduct", "confirm", "congress",
    "connect", "consider", "control", "convince", "cook", "cool", "copper", "copy", "coral",
    "core", "corn", "correct", "cost", "cotton", "couch", "country", "couple", "course", "cousin",
    "cover", "coyote", "crack", "cradle", "craft", "cram", "crane", "crash", "crater", "crawl",
    "crazy", "cream", "credit", "creek", "crew", "cricket", "crime", "crisp", "critic", "crop",
    "cross", "crouch", "crowd", "crucial", "cruel", "cruise", "crumble", "crunch", "crush", "cry",
    "crystal", "cube", "culture", "cup", "cupboard", "curious", "current", "curtain", "curve",
    "cushion", "custom", "cute", "cycle", "dad", "damage", "damp", "dance", "danger", "daring",
    "dash", "daughter", "dawn", "day", "deal", "debate", "debris", "decade", "december", "decide",
    "decline", "decorate", "decrease", "deer", "defense", "define", "defy", "degree", "delay",
    "deliver", "demand", "demise", "denial", "dentist", "deny", "depart", "depend", "deposit",
    "depth", "deputy", "derive", "describe", "desert", "design", "desk", "despair", "destroy",
    "detail", "detect", "develop", "device", "devote", "diagram", "dial", "diamond", "diary",
    "dice", "diesel", "diet", "differ", "digital", "dignity", "dilemma", "dinner", "dinosaur",
    "direct", "dirt", "disagree", "discover", "disease", "dish", "dismiss", "disorder", "display",
    "distance", "divert", "divide", "divorce", "dizzy", "doctor", "document", "dog", "doll",
    "dolphin", "domain", "donate", "donkey", "donor", "door", "dose", "double", "dove", "draft",
    "dragon", "drama", "drastic", "draw", "dream", "dress", "drift", "drill", "drink", "drip",
    "drive", "drop", "drum", "dry", "duck", "dumb", "dune", "during", "dust", "dutch", "duty",
    "dwarf", "dynamic", "eager", "eagle", "early", "earn", "earth", "easily", "east", "easy",
    "echo", "ecology", "economy", "edge", "edit", "educate", "effort", "egg", "eight", "either",
    "elbow", "elder", "electric", "elegant", "element", "elephant", "elevator", "elite", "else",
    "embark", "embody", "embrace", "emerge", "emotion", "employ", "empower", "empty", "enable",
    "enact", "end", "endless", "endorse", "enemy", "energy", "enforce", "engage", "engine",
    "enhance", "enjoy", "enlist", "enough", "enrich", "enroll", "ensure", "enter", "entire",
    "entry", "envelope", "episode", "equal", "equip", "era", "erase", "erode", "erosion", "error",
    "erupt", "escape", "essay", "essence", "estate", "eternal", "ethics", "evidence", "evil",
    "evoke", "evolve", "exact", "example", "excess", "exchange", "excite", "exclude", "excuse",
    "execute", "exercise", "exhaust", "exhibit", "exile", "exist", "exit", "exotic", "expand",
    "expect", "expire", "explain", "expose", "express", "extend", "extra", "eye", "eyebrow",
    "fabric", "face", "faculty", "fade", "faint", "faith", "fall", "false", "fame", "family",
    "famous", "fan", "fancy", "fantasy", "farm", "fashion", "fat", "fatal", "father", "fatigue",
    "fault", "favorite", "feature", "february", "federal", "fee", "feed", "feel", "female",
    "fence", "festival", "fetch", "fever", "few", "fiber", "fiction", "field", "figure", "file",
    "film", "filter", "final", "find", "fine", "finger", "finish", "fire", "firm", "first",
    "fiscal", "fish", "fit", "fitness", "fix", "flag", "flame", "flash", "flat", "flavor", "flee",
    "flight", "flip", "float", "flock", "floor", "flower", "fluid", "flush", "fly", "foam",
    "focus", "fog", "foil", "fold", "follow", "food", "foot", "force", "forest", "forget", "fork",
    "fortune", "forum", "forward", "fossil", "foster", "found", "fox", "fragile", "frame",
    "frequent", "fresh", "friend", "fringe", "frog", "front", "frost", "frown", "frozen", "fruit",
    "fuel", "fun", "funny", "furnace", "fury", "future", "gadget", "gain", "galaxy", "gallery",
    "game", "gap", "garage", "garbage", "garden", "garlic", "garment", "gas", "gasp", "gate",
    "gather", "gauge", "gaze", "general", "genius", "genre", "gentle", "genuine", "gesture",
    "ghost", "giant", "gift", "giggle", "ginger", "giraffe", "girl", "give", "glad", "glance",
    "glare", "glass", "glide", "glimpse", "globe", "gloom", "glory", "glove", "glow", "glue",
    "goat", "goddess", "gold", "good", "goose", "gorilla", "gospel", "gossip", "govern", "gown",
    "grab", "grace", "grain", "grant", "grape", "grass", "gravity", "great", "green", "grid",
    "grief", "grit", "grocery", "group", "grow", "grunt", "guard", "guess", "guide", "guilt",
    "guitar", "gun", "gym", "habit", "hair", "half", "hammer", "hamster", "hand", "happy",
    "harbor", "hard", "harsh", "harvest", "hat", "have", "hawk", "hazard", "head", "health",
    "heart", "heavy", "hedgehog", "height", "hello", "helmet", "help", "hen", "hero", "hidden",
    "high", "hill", "hint", "hip", "hire", "history", "hobby", "hockey", "hold", "hole", "holiday",
    "hollow", "home", "honey", "hood", "hope", "horn", "horror", "horse", "hospital", "host",
    "hotel", "hour", "hover", "hub", "huge", "human", "humble", "humor", "hundred", "hungry",
    "hunt", "hurdle", "hurry", "hurt", "husband", "hybrid", "ice", "icon", "idea", "identify",
    "idle", "ignore", "ill", "illegal", "illness", "image", "imitate", "immense", "immune",
    "impact", "impose", "improve", "impulse", "inch", "include", "income", "increase", "index",
    "indicate", "indoor", "industry", "infant", "inflict", "inform", "inhale", "inherit",
    "initial", "inject", "injury", "inmate", "inner", "innocent", "input", "inquiry", "insane",
    "insect", "inside", "inspire", "install", "intact", "interest", "into", "invest", "invite",
    "involve", "iron", "island", "isolate", "issue", "item", "ivory", "jacket", "jaguar", "jar",
    "jazz", "jealous", "jeans", "jelly", "jewel", "job", "join", "joke", "journey", "joy", "judge",
    "juice", "jump", "jungle", "junior", "junk", "just", "kangaroo", "keen", "keep", "ketchup",
    "key", "kick", "kid", "kidney", "kind", "kingdom", "kiss", "kit", "kitchen", "kite", "kitten",
    "kiwi", "knee", "knife", "knock", "know", "lab", "label", "labor", "ladder", "lady", "lake",
    "lamp", "language", "laptop", "large", "later", "latin", "laugh", "laundry", "lava", "law",
    "lawn", "lawsuit", "layer", "lazy", "leader", "leaf", "learn", "leave", "lecture", "left",
    "leg", "legal", "legend", "leisure", "lemon", "lend", "length", "lens", "leopard", "lesson",
    "letter", "level", "liar", "liberty", "library", "license", "life", "lift", "light", "like",
    "limb", "limit", "link", "lion", "liquid", "list", "little", "live", "lizard", "load", "loan",
    "lobster", "local", "lock", "logic", "lonely", "long", "loop", "lottery", "loud", "lounge",
    "love", "loyal", "lucky", "luggage", "lumber", "lunar", "lunch", "luxury", "lyrics", "machine",
    "mad", "magic", "magnet", "maid", "mail", "main", "major", "make", "mammal", "man", "manage",
    "mandate", "mango", "mansion", "manual", "maple", "marble", "march", "margin", "marine",
    "market", "marriage", "mask", "mass", "master", "match", "material", "math", "matrix",
    "matter", "maximum", "maze", "meadow", "mean", "measure", "meat", "mechanic", "medal", "media",
    "melody", "melt", "member", "memory", "mention", "menu", "mercy", "merge", "merit", "merry",
    "mesh", "message", "metal", "method", "middle", "midnight", "milk", "million", "mimic", "mind",
    "minimum", "minor", "minute", "miracle", "mirror", "misery", "miss", "mistake", "mix", "mixed",
    "mixture", "mobile", "model", "modify", "mom", "moment", "monitor", "monkey", "monster",
    "month", "moon", "moral", "more", "morning", "mosquito", "mother", "motion", "motor",
    "mountain", "mouse", "move", "movie", "much", "muffin", "mule", "multiply", "muscle", "museum",
    "mushroom", "music", "must", "mutual", "myself", "mystery", "myth", "naive", "name", "napkin",
    "narrow", "nasty", "nation", "nature", "near", "neck", "need", "negative", "neglect",
    "neither", "nephew", "nerve", "nest", "net", "network", "neutral", "never", "news", "next",
    "nice", "night", "noble", "noise", "nominee", "noodle", "normal", "north", "nose", "notable",
    "note", "nothing", "notice", "novel", "now", "nuclear", "number", "nurse", "nut", "oak",
    "obey", "object", "oblige", "obscure", "observe", "obtain", "obvious", "occur", "ocean",
    "october", "odor", "off", "offer", "office", "often", "oil", "okay", "old", "olive", "olympic",
    "omit", "once", "one", "onion", "online", "only", "open", "opera", "opinion", "oppose",
    "option", "orange", "orbit", "orchard", "order", "ordinary", "organ", "orient", "original",
    "orphan", "ostrich", "other", "outdoor", "outer", "output", "outside", "oval", "oven", "over",
    "own", "owner", "oxygen", "oyster", "ozone", "pact", "paddle", "page", "pair", "palace",
    "palm", "panda", "panel", "panic", "panther", "paper", "parade", "parent", "park", "parrot",
    "party", "pass", "patch", "path", "patient", "patrol", "pattern", "pause", "pave", "payment",
    "peace", "peanut", "pear", "peasant", "pelican", "pen", "penalty", "pencil", "people",
    "pepper", "perfect", "permit", "person", "pet", "phone", "photo", "phrase", "physical",
    "piano", "picnic", "picture", "piece", "pig", "pigeon", "pill", "pilot", "pink", "pioneer",
    "pipe", "pistol", "pitch", "pizza", "place", "planet", "plastic", "plate", "play", "please",
    "pledge", "pluck", "plug", "plunge", "poem", "poet", "point", "polar", "pole", "police",
    "pond", "pony", "pool", "popular", "portion", "position", "possible", "post", "potato",
    "pottery", "poverty", "powder", "power", "practice", "praise", "predict", "prefer", "prepare",
    "present", "pretty", "prevent", "price", "pride", "primary", "print", "priority", "prison",
    "private", "prize", "problem", "process", "produce", "profit", "program", "project", "promote",
    "proof", "property", "prosper", "protect", "proud", "provide", "public", "pudding", "pull",
    "pulp", "pulse", "pumpkin", "punch", "pupil", "puppy", "purchase", "purity", "purpose",
    "purse", "push", "put", "puzzle", "pyramid", "quality", "quantum", "quarter", "question",
    "quick", "quit", "quiz", "quote", "rabbit", "raccoon", "race", "rack", "radar", "radio",
    "rail", "rain", "raise", "rally", "ramp", "ranch", "random", "range", "rapid", "rare", "rate",
    "rather", "raven", "raw", "razor", "ready", "real", "reason", "rebel", "rebuild", "recall",
    "receive", "recipe", "record", "recycle", "reduce", "reflect", "reform", "refuse", "region",
    "regret", "regular", "reject", "relax", "release", "relief", "rely", "remain", "remember",
    "remind", "remove", "render", "renew", "rent", "reopen", "repair", "repeat", "replace",
    "report", "require", "rescue", "resemble", "resist", "resource", "response", "result",
    "retire", "retreat", "return", "reunion", "reveal", "review", "reward", "rhythm", "rib",
    "ribbon", "rice", "rich", "ride", "ridge", "rifle", "right", "rigid", "ring", "riot", "ripple",
    "risk", "ritual", "rival", "river", "road", "roast", "robot", "robust", "rocket", "romance",
    "roof", "rookie", "room", "rose", "rotate", "rough", "round", "route", "royal", "rubber",
    "rude", "rug", "rule", "run", "runway", "rural", "sad", "saddle", "sadness", "safe", "sail",
    "salad", "salmon", "salon", "salt", "salute", "same", "sample", "sand", "satisfy", "satoshi",
    "sauce", "sausage", "save", "say", "scale", "scan", "scare", "scatter", "scene", "scheme",
    "school", "science", "scissors", "scorpion", "scout", "scrap", "screen", "script", "scrub",
    "sea", "search", "season", "seat", "second", "secret", "section", "security", "seed", "seek",
    "segment", "select", "sell", "seminar", "senior", "sense", "sentence", "series", "service",
    "session", "settle", "setup", "seven", "shadow", "shaft", "shallow", "share", "shed", "shell",
    "sheriff", "shield", "shift", "shine", "ship", "shiver", "shock", "shoe", "shoot", "shop",
    "short", "shoulder", "shove", "shrimp", "shrug", "shuffle", "shy", "sibling", "sick", "side",
    "siege", "sight", "sign", "silent", "silk", "silly", "silver", "similar", "simple", "since",
    "sing", "siren", "sister", "situate", "six", "size", "skate", "sketch", "ski", "skill", "skin",
    "skirt", "skull", "slab", "slam", "sleep", "slender", "slice", "slide", "slight", "slim",
    "slogan", "slot", "slow", "slush", "small", "smart", "smile", "smoke", "smooth", "snack",
    "snake", "snap", "sniff", "snow", "soap", "soccer", "social", "sock", "soda", "soft", "solar",
    "soldier", "solid", "solution", "solve", "someone", "song", "soon", "sorry", "sort", "soul",
    "sound", "soup", "source", "south", "space", "spare", "spatial", "spawn", "speak", "special",
    "speed", "spell", "spend", "sphere", "spice", "spider", "spike", "spin", "spirit", "split",
    "spoil", "sponsor", "spoon", "sport", "spot", "spray", "spread", "spring", "spy", "square",
    "squeeze", "squirrel", "stable", "stadium", "staff", "stage", "stairs", "stamp", "stand",
    "start", "state", "stay", "steak", "steel", "stem", "step", "stereo", "stick", "still",
    "sting", "stock", "stomach", "stone", "stool", "story", "stove", "strategy", "street",
    "strike", "strong", "struggle", "student", "stuff", "stumble", "style", "subject", "submit",
    "subway", "success", "such", "sudden", "suffer", "sugar", "suggest", "suit", "summer", "sun",
    "sunny", "sunset", "super", "supply", "supreme", "sure", "surface", "surge", "surprise",
    "surround", "survey", "suspect", "sustain", "swallow", "swamp", "swap", "swarm", "swear",
    "sweet", "swift", "swim", "swing", "switch", "sword", "symbol", "symptom", "syrup", "system",
    "table", "tackle", "tag", "tail", "talent", "talk", "tank", "tape", "target", "task", "taste",
    "tattoo", "taxi", "teach", "team", "tell", "ten", "tenant", "tennis", "tent", "term", "test",
    "text", "thank", "that", "theme", "then", "theory", "there", "they", "thing", "this",
    "thought", "three", "thrive", "throw", "thumb", "thunder", "ticket", "tide", "tiger", "tilt",
    "timber", "time", "tiny", "tip", "tired", "tissue", "title", "toast", "tobacco", "today",
    "toddler", "toe", "together", "toilet", "token", "tomato", "tomorrow", "tone", "tongue",
    "tonight", "tool", "tooth", "top", "topic", "topple", "torch", "tornado", "tortoise", "toss",
    "total", "tourist", "toward", "tower", "town", "toy", "track", "trade", "traffic", "tragic",
    "train", "transfer", "trap", "trash", "travel", "tray", "treat", "tree", "trend", "trial",
    "tribe", "trick", "trigger", "trim", "trip", "trophy", "trouble", "truck", "true", "truly",
    "trumpet", "trust", "truth", "try", "tube", "tuition", "tumble", "tuna", "tunnel", "turkey",
    "turn", "turtle", "twelve", "twenty", "twice", "twin", "twist", "two", "type", "typical",
    "ugly", "umbrella", "unable", "unaware", "uncle", "uncover", "under", "undo", "unfair",
    "unfold", "unhappy", "uniform", "unique", "unit", "universe", "unknown", "unlock", "until",
    "unusual", "unveil", "update", "upgrade", "uphold", "upon", "upper", "upset", "urban", "urge",
    "usage", "use", "used", "useful", "useless", "usual", "utility", "vacant", "vacuum", "vague",
    "valid", "valley", "valve", "van", "vanish", "vapor", "various", "vast", "vault", "vehicle",
    "velvet", "vendor", "venture", "venue", "verb", "verify", "version", "very", "vessel",
    "veteran", "viable", "vibrant", "vicious", "victory", "video", "view", "village", "vintage",
    "violin", "virtual", "virus", "visa", "visit", "visual", "vital", "vivid", "vocal", "voice",
    "void", "volcano", "volume", "vote", "voyage", "wage", "wagon", "wait", "walk", "wall",
    "walnut", "want", "warfare", "warm", "warrior", "wash", "wasp", "waste", "water", "wave",
    "way", "wealth", "weapon", "wear", "weasel", "weather", "web", "wedding", "weekend", "weird",
    "welcome", "west", "wet", "whale", "what", "wheat", "wheel", "when", "where", "whip",
    "whisper", "wide", "width", "wife", "wild", "will", "win", "window", "wine", "wing", "wink",
    "winner", "winter", "wire", "wisdom", "wise", "wish", "witness", "wolf", "woman", "wonder",
    "wood", "wool", "word", "work", "world", "worry", "worth", "wrap", "wreck", "wrestle", "wrist",
    "write", "wrong", "yard", "year", "yellow", "you", "young", "youth", "zebra", "zero", "zone",
    "zoo",
];
