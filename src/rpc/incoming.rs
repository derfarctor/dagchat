use super::{blockinfo::get_blocks_info, process::*};
use crate::app::{
    components::{
        messages::structs::Message,
        receive::structs::{Receivable, ReceivableRequest, ReceivableResponse},
    },
    constants::REQ_TIMEOUT,
};
use crate::crypto::keys::to_public_key;
use cursive::utils::Counter;

pub fn find_incoming(
    target_address: &str,
    node_url: &str,
    counter: &Counter,
) -> Result<Vec<Receivable>, String> {
    let request = ReceivableRequest {
        action: String::from("pending"),
        account: String::from(target_address),
        count: String::from("50"),
        source: true,
    };

    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url, REQ_TIMEOUT)?;
    counter.tick(200);

    let receivables: Result<ReceivableResponse, serde_json::error::Error> =
        serde_json::from_str(&response);
    let receivables = match receivables {
        Ok(receivables) => receivables,
        // If deserialisation failed, either there were no blocks
        // Or an different error was encountered.
        Err(error) => {
            // If the error was missing the blocks field, then it
            // likely wasn't due to deserialising a response returned
            // as a result of having no receivables; it was instead a network error.
            if error.to_string().contains("missing field") {
                return Err(error.to_string() + ": " + &response);
            } else {
                return Ok(vec![]);
            }
        }
    };

    let receivable_blocks = receivables.blocks.data;
    let mut head_hashes: Vec<String> = vec![];
    for block in &receivable_blocks {
        head_hashes.push(block.0.clone());
    }
    counter.tick(50);
    let head_blocks_info = get_blocks_info(head_hashes, node_url)?;
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
    let root_blocks_info = get_blocks_info(root_hashes, node_url)?;
    counter.tick(100);
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
    Ok(incoming)
}
