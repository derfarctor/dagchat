use crate::app::coin::Coin;
use crate::crypto::{
    address::get_address,
    blocks::{get_block_hash, get_signed_block},
    conversions::get_32_bytes,
    keys::to_public_key,
};
use crate::rpc::accountinfo::{get_account_info, get_balance};
use crate::rpc::process::publish_block;
use cursive::utils::Counter;
pub fn send(
    private_key_bytes: &[u8; 32],
    address: String,
    raw: u128,
    coin: &Coin,
    counter: &Counter,
) -> Result<String, String> {
    // Derive sender's address
    let sender_pub = ed25519_dalek::PublicKey::from(
        &ed25519_dalek::SecretKey::from_bytes(private_key_bytes).unwrap(),
    );
    let sender_address = get_address(sender_pub.as_bytes(), Some(&coin.prefix));

    // Safe because account must be opened to have got this far
    let account_info = get_account_info(&sender_address, &coin.network.node_url)?;

    let last_block_hash = get_32_bytes(&account_info.frontier);
    let new_balance = get_balance(&account_info) - raw;
    let representative = to_public_key(&account_info.representative);
    let link = to_public_key(&address);
    counter.tick(200);
    let sub = String::from("send");
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
    counter.tick(500);
    publish_block(signed_block, sub, &coin.network)
}
