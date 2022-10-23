use crate::crypto::keys::to_public_key;
use crate::rpc::{blockinfo::Block, history::get_history};

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
        return String::from("Error decrypting message: not sent using the dagchat protocol.");
    }
    let plaintext = String::from_utf8(decrypted.unwrap());
    if plaintext.is_err() {
        return String::from("Error decrypting message: format was not UTF-8.");
    }
    //println!("{}", plaintext.unwrap());
    plaintext.unwrap()
}

pub fn extract_message(blocks: Vec<Block>) -> Vec<u8> {
    let mut encrypted_bytes = vec![];
    for block in blocks {
        let block_data = to_public_key(&block.representative);
        encrypted_bytes.extend(&block_data);
    }
    encrypted_bytes
}
