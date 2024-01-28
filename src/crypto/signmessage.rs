use super::blocks::{get_block_hash, get_signed_block};
use crate::app::coin::Coin;
use crate::app::constants::{BANANO_MESSAGE_PREAMBLE, NANO_MESSAGE_PREAMBLE};
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;

pub fn sign_message(private_key_bytes: &[u8; 32], message: &str, coin: &Coin) -> Result<String, String> {
    let mut hasher = Blake2bVar::new(32).unwrap();
    let mut message_encoded_rep_buf = [0u8; 32];
    if coin.name == "nano" {
        hasher.update(NANO_MESSAGE_PREAMBLE);
    } else if coin.name == "banano" {
        hasher.update(BANANO_MESSAGE_PREAMBLE);
    }
    hasher.update(message.as_bytes());
    hasher.finalize_variable(&mut message_encoded_rep_buf).unwrap();
    let block_hash = get_block_hash(
        private_key_bytes,
        &message_encoded_rep_buf,
        &[0; 32],
        &[0; 32],
        0
    );
    let signed_block = get_signed_block(
        private_key_bytes,
        //hashed message goes into rep field
        &message_encoded_rep_buf,
        &[0; 32],
        &[0; 32],
        0,
        &block_hash,
        coin,
        "change"
    )?;
    Ok(signed_block.signature)
}
