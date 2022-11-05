use super::accountinfo::{get_balance, AccountInfoResponse};
use super::process::publish_block;
use crate::app::coin::Coin;
use crate::crypto::blocks::{get_block_hash, get_signed_block};
use crate::crypto::conversions::get_32_bytes;
use crate::crypto::keys::to_public_key;

pub fn change_rep(
    private_key_bytes: &[u8; 32],
    account_info: AccountInfoResponse,
    rep_address: &str,
    coin: &Coin,
) -> Result<String, String> {
    let last_block_hash = get_32_bytes(&account_info.frontier);
    let balance = get_balance(&account_info);
    let representative = to_public_key(rep_address);
    let link = [0u8; 32];
    let sub = String::from("change");
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
    )?;
    publish_block(block, sub, &coin.network)
}
