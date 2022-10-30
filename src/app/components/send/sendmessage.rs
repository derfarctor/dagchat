use crate::app::coin::Coin;
use crate::crypto::{
    address::get_address, blocks::*, conversions::get_32_bytes, keys::to_public_key,
};
use crate::rpc::{accountinfo::*, process::publish_block};
use cursive::utils::Counter;

pub fn send_message(
    private_key_bytes: &[u8; 32],
    target_address: String,
    raw: u128,
    mut message: String,
    coin: &Coin,
    counter: &Counter,
) -> String {
    let public_key_bytes = to_public_key(&target_address);
    let pad = (message.len() + 28) % 32;
    for _ in 0..(32 - pad) {
        message.push(' ');
    }
    let public_key = ecies_ed25519::PublicKey::from_bytes(&public_key_bytes).unwrap();

    let mut csprng = rand::thread_rng();
    let encrypted_bytes =
        ecies_ed25519::encrypt(&public_key, message.as_bytes(), &mut csprng).unwrap();
    let blocks_needed = ((60 + message.len()) / 32) + 1;

    let mut block_data = [0u8; 32];
    let mut first_block_hash = [0u8; 32];

    // Derive sender's address
    let sender_pub = ed25519_dalek::PublicKey::from(
        &ed25519_dalek::SecretKey::from_bytes(private_key_bytes).unwrap(),
    );
    let sender_address = get_address(sender_pub.as_bytes(), Some(&coin.prefix));

    // Set up the previous block hash and balance to start publishing blocks
    // Also note the representative from before sending, in order to change back afterwards
    let account_info = get_account_info(&sender_address, &coin.network.node_url).unwrap();
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
            coin,
            &sub,
        );
        if block_num == 0 {
            first_block_hash = block_hash;
        }
        last_block_hash = block_hash;
        publish_block(block, sub.clone(), &coin.network.node_url);
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
        coin,
        &sub,
    );
    publish_block(block, sub, &coin.network.node_url);
    hex::encode(last_block_hash)
}
