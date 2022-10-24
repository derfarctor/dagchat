use crate::app::constants::{banano, nano};
use crate::crypto::{
    blocks::{get_block_hash, get_signed_block},
    conversions::get_32_bytes,
    keys::to_public_key,
};
use crate::rpc::{accountinfo::*, process::*};
use cursive::utils::Counter;

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

    if let Ok(account_info) = account_info_opt {
        last_block_hash = get_32_bytes(&account_info.frontier);
        let balance = get_balance(&account_info);
        new_balance = balance + amount;
        representative = to_public_key(&account_info.representative);
    } else {
        // OPEN BLOCK
        if addr_prefix == "nano_" {
            representative = to_public_key(nano::DEFAULT_REP);
        } else if addr_prefix == "ban_" {
            representative = to_public_key(banano::DEFAULT_REP);
        } else {
            panic!("Unknown network... no default rep to open account.");
        }
    }

    counter.tick(200);
    let sub = String::from("receive");
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
        &sub,
    );
    counter.tick(200);
    publish_block(signed_block, sub, node_url);
    counter.tick(200);
}
