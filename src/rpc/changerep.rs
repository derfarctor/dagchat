use super::accountinfo::{get_balance, AccountInfoResponse};
use super::process::publish_block;
use crate::crypto::blocks::{get_block_hash, get_signed_block};
use crate::crypto::conversions::get_32_bytes;
use crate::crypto::keys::to_public_key;

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
