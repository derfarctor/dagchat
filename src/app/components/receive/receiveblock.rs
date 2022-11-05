use crate::app::coin::Coin;
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
    coin: &Coin,
    counter: &Counter,
) -> Result<String, String> {
    let account_info_opt = get_account_info(address, &coin.network.node_url);
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
        representative = to_public_key(&coin.network.default_rep);
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
        coin,
        &sub,
    )?;
    counter.tick(400);
    publish_block(signed_block, sub, &coin.network)
}
