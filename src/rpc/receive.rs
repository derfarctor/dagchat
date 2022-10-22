use super::{accountinfo::*, blockinfo::get_blocks_info, process::*};
use crate::app::{
    components::messages::structs::Message,
    constants::{DEFAULT_REP_BANANO, DEFAULT_REP_NANO},
};
use crate::crypto::{
    blocks::{get_block_hash, get_signed_block},
    conversions::get_32_bytes,
    keys::to_public_key,
};
use cursive::utils::Counter;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceivableRequest {
    pub action: String,
    pub account: String,
    pub source: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceivableResponse {
    pub blocks: ReceivableBlocks,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceivableBlocks {
    #[serde(flatten)]
    pub data: HashMap<String, ReceivableBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceivableBlock {
    pub amount: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Receivable {
    pub hash: String,
    pub message: Option<Message>,
    pub amount: u128,
    // Used for seeing message sender in app
    pub source: String,
}

pub fn find_incoming(target_address: &str, node_url: &str, counter: &Counter) -> Vec<Receivable> {
    let request = ReceivableRequest {
        action: String::from("pending"),
        account: String::from(target_address),
        source: true,
    };

    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url);
    counter.tick(200);

    //eprintln!("{}", &response);
    let receivables: Result<ReceivableResponse, _> = serde_json::from_str(&response);
    let receivables = match receivables {
        Ok(receivables) => receivables,
        // If deserialisation failed, either there were no blocks
        // Or an different error was encountered.
        Err(_) => return vec![],
    };

    let receivable_blocks = receivables.blocks.data;
    let mut head_hashes: Vec<String> = vec![];
    for block in &receivable_blocks {
        head_hashes.push(block.0.clone());
    }
    counter.tick(50);

    let head_blocks_info = get_blocks_info(head_hashes, node_url);
    counter.tick(200);
    let mut raw_head_blocks = head_blocks_info.blocks.data;
    let mut root_hashes: Vec<String> = vec![];

    for block in &raw_head_blocks {
        let rep = &block.1.contents.representative;
        let bytes = to_public_key(rep);
        let hash = hex::encode(bytes);
        root_hashes.push(hash);
    }
    counter.tick(50);
    let root_blocks_info = get_blocks_info(root_hashes, node_url);
    counter.tick(200);
    let raw_root_blocks = root_blocks_info.blocks.data;

    let mut incoming: Vec<Receivable> = vec![];
    let x = 200usize / receivable_blocks.len();
    for receivable in receivable_blocks {
        let head_block = raw_head_blocks.remove(&receivable.0).unwrap();
        let hash = hex::encode(to_public_key(&head_block.contents.representative));
        let mut message: Option<Message> = None;
        if raw_root_blocks.contains_key(&hash) {
            let root_block = raw_root_blocks.get(&hash).unwrap();
            let head_height: u64 = head_block.height.parse().unwrap();
            let root_height: u64 = root_block.height.parse().unwrap();
            let message_block_count = head_height - root_height;
            message = Some(Message {
                blocks: message_block_count,
                head: head_block,
                root_hash: hash,
                plaintext: String::from(""),
            });
        }
        incoming.push(Receivable {
            hash: receivable.0,
            amount: receivable.1.amount.parse().unwrap(),
            source: receivable.1.source,
            message,
        });
        counter.tick(x);
    }
    incoming
}

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

    if let Some(account_info) = account_info_opt {
        last_block_hash = get_32_bytes(&account_info.frontier);
        let balance = get_balance(&account_info);
        new_balance = balance + amount;
        representative = to_public_key(&account_info.representative);
    } else {
        // OPEN BLOCK
        if addr_prefix == "nano_" {
            representative = to_public_key(DEFAULT_REP_NANO);
        } else if addr_prefix == "ban_" {
            representative = to_public_key(DEFAULT_REP_BANANO);
        } else {
            panic!("Unknown network... no default rep to open account.");
        }
    }

    counter.tick(200);
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
    );
    counter.tick(200);
    publish_block(signed_block, String::from("receive"), node_url);
    counter.tick(200);
}
