use super::{address::get_address, pow::*};
use crate::app::coin::Coin;
use crate::app::components::settings::structs::WorkType;
use crate::rpc::blockinfo::Block;
use crate::rpc::workgenerate::get_server_work;
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
    coin: &Coin,
    sub: &str,
) -> Result<Block, String> {
    let secret = ed25519_dalek::SecretKey::from_bytes(priv_k).unwrap();
    let public = ed25519_dalek::PublicKey::from(&secret);
    let expanded_secret = ed25519_dalek::ExpandedSecretKey::from(&secret);

    let internal_signed =
        expanded_secret.sign(block_hash, &ed25519_dalek::PublicKey::from(&secret));
    let signed_bytes = internal_signed.to_bytes();

    let work_type = coin.network.work_type;
    let work = if work_type == WorkType::CPU || work_type == WorkType::WORK_SERVER {
        // If it is the open block then use the public key to generate work.
        // If not, use previous block hash.
        let mut previous_hash = previous;
        if previous_hash == &[0u8; 32] {
            previous_hash = public.as_bytes();
        }

        if work_type == WorkType::CPU {
            let threshold = if sub == "receive" {
                u64::from_str_radix(&coin.network.receive_thresh, 16).unwrap()
            } else {
                u64::from_str_radix(&coin.network.send_thresh, 16).unwrap()
            };
            generate_work(previous_hash, threshold)
        } else {
            let threshold = if sub == "receive" {
                &coin.network.receive_thresh
            } else {
                &coin.network.send_thresh
            };

            get_server_work(previous_hash, threshold, &coin.network.work_server_url)?
        }
    } else if work_type == WorkType::BOOMPOW {
        String::from("")
    } else {
        panic!("Unknown network WorkType.");
    };

    let block = Block {
        type_name: String::from("state"),
        account: get_address(public.as_bytes(), Some(&coin.prefix)),
        previous: hex::encode(previous),
        representative: get_address(rep, Some(&coin.prefix)),
        balance: balance.to_string(),
        link: hex::encode(link),
        work,
        signature: hex::encode(&signed_bytes),
    };
    Ok(block)
}
