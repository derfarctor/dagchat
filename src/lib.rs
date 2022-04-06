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

const ADDR_ENCODING: Encoding = new_encoding! {
    symbols: "13456789abcdefghijkmnopqrstuwxyz",
    check_trailing_bits: false,
};

const PREFIX: &str = "ban_";

//nano
//https://app.natrium.io/api

//banano
//https://kaliumapi.appditto.com/api

struct MessageResult {
    success: bool,
    blocks: u64,
    msg: String,
    time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountInfoResponse {
    frontier: String,
    open_block: String,
    representative_block: String,
    balance: String,
    modified_timestamp: String,
    block_count: String,
    account_version: String,
    confirmation_height: String,
    confirmation_height_frontier: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default)]
    account: String,
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

pub struct MessageRoot {
    pub root: String,
    // Used for seeing message sender in app
    source: String,
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
    contents: Block,
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
    message: String,
    node_url: &str,
) {
    let public_key_bytes = to_public_key(target_address);
    let mut message = message.clone();
    let pad = (message.len() + 28) % 32;
    for _ in 0..(32 - pad) {
        message.push_str(" ");
    }
    let public_key = ecies_ed25519::PublicKey::from_bytes(&public_key_bytes).unwrap();
    println!("Encrypting message for send: {}", message);
    let mut csprng = rand::thread_rng();
    let encrypted_bytes =
        ecies_ed25519::encrypt(&public_key, message.as_bytes(), &mut csprng).unwrap();
    let blocks_needed = ((60 + message.len()) / 32) + 1;
    println!("Blocks needed: {}", blocks_needed);

    let mut block_data = [0u8; 32];
    let mut first_block_hash = [0u8; 32];

    // Derive sender's address
    let sender_pub = ed25519_dalek::PublicKey::from(
        &ed25519_dalek::SecretKey::from_bytes(private_key_bytes).unwrap(),
    );
    let sender_address = get_address(sender_pub.as_bytes());

    // Set up the previous block hash and balance to start publishing blocks
    let (mut last_block_hash, mut balance) = get_frontier_and_balance(sender_address, node_url);
    let mut link = [0u8; 32];
    let mut sub = String::from("change");

    for block_num in 0..blocks_needed {
        let start = 32 * block_num;
        let end = 32 * (block_num + 1);
        if block_num == blocks_needed - 1 {
            // Last block sent is the send block with 1 raw to recipient
            // Link is the recipient
            // Rep is the hash of the first block in the message
            block_data = first_block_hash;
            balance -= 1;
            link = public_key_bytes;
            sub = String::from("send");
        } else {
            block_data.copy_from_slice(&encrypted_bytes[start..end]);
        }
        println!("Block data as addr: {:?}", get_address(&block_data));
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
        );
        if block_num == 0 {
            first_block_hash = block_hash;
        }
        last_block_hash = block_hash;
        //println!("{}", block.to_string());
        let hash = publish_block(block, sub.clone(), node_url);
        println!("{}", hash);
    }
}

pub fn read_message(
    private_key_bytes: &[u8; 32],
    start_block_hash: String,
    node_url: &str,
) -> String {
    let root_block = get_block_info(start_block_hash.clone(), node_url);
    let root_height: u64 = root_block.height.parse().unwrap();
    let representative_bytes = to_public_key(root_block.contents.representative);
    let message_root_hash = hex::encode(representative_bytes);
    // Since block is already pulled, can refrain from requesting it again in get_history
    // Can be implemented with offset 1
    let message_root_block = get_block_info(message_root_hash.clone(), node_url);
    let message_height: u64 = message_root_block.height.parse().unwrap();
    let message_block_count = root_height - message_height;
    println!("Message length in blocks: {}", message_block_count);
    let target = root_block.contents.account;
    let message_blocks = get_history(target, message_root_hash, message_block_count, node_url);

    let encrypted_bytes = extract_message(message_blocks);

    let dalek = ed25519_dalek::SecretKey::from_bytes(private_key_bytes).unwrap();
    let expanded_bytes = ed25519_dalek::ExpandedSecretKey::from(&dalek);
    let private_key =
        ecies_ed25519::SecretKey::from_bytes(&expanded_bytes.to_bytes()[0..32]).unwrap();
    let decrypted = ecies_ed25519::decrypt(&private_key, &encrypted_bytes).unwrap();
    let plaintext = String::from_utf8(decrypted).unwrap();
    println!("{}", plaintext);
    plaintext
}

pub fn find_incoming(target_address: String, node_url: &str) -> Vec<MessageRoot> {
    let request = ReceivableRequest {
        action: String::from("pending"),
        account: target_address,
        source: true,
        /* Maybe there's an efficient way to use this working backwards. Not implemented yet. */
        sorting: true,
    };

    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url);
    //println!("{}", response);
    let receivable: ReceivableResponse = serde_json::from_str(&response).unwrap();

    let blocks = receivable.blocks.data;
    let mut incoming: Vec<MessageRoot> = vec![];
    for block in blocks {
        //println!("{} info: {:?}", block.0, block.1);
        if block.1.amount == "1" {
            incoming.push(MessageRoot {
                root: block.0,
                source: block.1.source,
            });
        }
    }
    incoming
}

pub fn get_block_info(hash: String, node_url: &str) -> BlockResponse {
    let request = BlockRequest {
        action: String::from("block_info"),
        json_block: true,
        hash: hash,
    };

    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url);
    //println!("{}", response);
    let block_info: BlockResponse = serde_json::from_str(&response).unwrap();
    block_info
}

pub fn get_frontier_and_balance(address: String, node_url: &str) -> ([u8; 32], u128) {
    let body_json = json!({
        "action": "account_info",
        "account": address
    });
    let body = body_json.to_string();
    let resp_string = post_node(body, node_url);
    //println!("Raw response: {}", resp_string);
    let account_info: AccountInfoResponse = serde_json::from_str(&resp_string).unwrap();
    let frontier_bytes = hex::decode(&account_info.frontier).unwrap();
    let mut frontier = [0u8; 32];
    frontier.copy_from_slice(frontier_bytes.as_slice());
    let balance: u128 = account_info.balance.parse().unwrap();
    (frontier, balance)
}

pub fn get_history(
    target_address: String,
    head: String,
    length: u64,
    node_url: &str,
) -> Vec<Block> {
    let request = HistoryRequest {
        action: String::from("account_history"),
        account: target_address,
        count: length,
        head: head,
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
        let block_data = to_public_key(block.representative);
        encrypted_bytes.extend(&block_data);
    }
    encrypted_bytes
}

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
        println!("Successfully communicated with node");
        let response_str = res.text().unwrap();
        return response_str;
    } else {
        println!("Issue. Status: {}", res.status());
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
        account: get_address(public.as_bytes()),
        previous: hex::encode(previous),
        representative: get_address(rep),
        balance: balance.to_string(),
        link: hex::encode(link),
        signature: z,
    };

    block
}

pub fn validate_mnemonic(mnemonic: &str) -> ([u8; 32], bool) {
    let (mnemonic, valid) = get_num_equivalent(mnemonic);
    if !valid {
        return ([0u8; 32], false);
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
            return (entropy, false);
        }
    }

    (entropy, true)
}

pub fn get_private_key(seed_bytes: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Blake2bVar::new(32).unwrap();
    let mut buf = [0u8; 32];
    hasher.update(seed_bytes);
    hasher.update(&[0u8; 4]);
    hasher.finalize_variable(&mut buf).unwrap();
    buf
}

pub fn get_address(pub_key_bytes: &[u8]) -> String {
    let mut pub_key_vec = pub_key_bytes.to_vec();
    let mut h = [0u8; 3].to_vec();
    h.append(&mut pub_key_vec);
    let checksum = ADDR_ENCODING.encode(&compute_address_checksum(pub_key_bytes));
    let address = {
        let encoded_addr = ADDR_ENCODING.encode(&h);
        let mut addr = String::from(PREFIX);
        addr.push_str(encoded_addr.get(4..).unwrap());
        addr.push_str(&checksum);
        addr
    };
    address
}

pub fn to_public_key(addr: String) -> [u8; 32] {
    let mut encoded_addr: String;
    let parts: Vec<&str> = addr.split("_").collect();
    encoded_addr = String::from(parts[1].get(0..52).unwrap());
    encoded_addr.insert_str(0, "1111");
    let mut pub_key_vec = ADDR_ENCODING.decode(encoded_addr.as_bytes()).unwrap();
    pub_key_vec.drain(0..3);
    pub_key_vec.as_slice().try_into().unwrap()
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
