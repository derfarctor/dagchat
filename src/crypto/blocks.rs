use super::{address::get_address, pow::*};

use crate::app::constants::{banano, nano, LOCAL_WORK};
use crate::rpc::blockinfo::Block;
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;

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
    while x.len() < 32 {
        x = format!("0{}", x);
    }
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
    sub: &str,
) -> Block {
    let secret = ed25519_dalek::SecretKey::from_bytes(priv_k).unwrap();
    let public = ed25519_dalek::PublicKey::from(&secret);
    let expanded_secret = ed25519_dalek::ExpandedSecretKey::from(&secret);

    let internal_signed =
        expanded_secret.sign(block_hash, &ed25519_dalek::PublicKey::from(&secret));
    let signed_bytes = internal_signed.to_bytes();

    let work = if LOCAL_WORK {
        // If it is the open block then use the public key to generate work.
        // If not, use previous block hash.
        let mut previous_hash = previous;
        if previous_hash == &[0u8; 32] {
            previous_hash = public.as_bytes();
        }
        let threshold;
        if addr_prefix == "nano_" {
            if sub == "receive" {
                threshold = u64::from_str_radix(nano::RECEIVE_THRESH, 16).unwrap();
            } else {
                threshold = u64::from_str_radix(nano::SEND_THRESH, 16).unwrap();
            }
        } else if addr_prefix == "ban_" {
            if sub == "receive" {
                threshold = u64::from_str_radix(banano::RECEIVE_THRESH, 16).unwrap();
            } else {
                threshold = u64::from_str_radix(banano::SEND_THRESH, 16).unwrap();
            }
        } else {
            panic!("Unknown coin threshold");
        }
        generate_work(previous_hash, threshold)
    } else {
        String::from("")
    };

    let block = Block {
        type_name: String::from("state"),
        account: get_address(public.as_bytes(), Some(addr_prefix)),
        previous: hex::encode(previous),
        representative: get_address(rep, Some(addr_prefix)),
        balance: balance.to_string(),
        link: hex::encode(link),
        work,
        signature: hex::encode(&signed_bytes),
    };

    block
}
