use super::process::post_node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub account: String,
    pub previous: String,
    pub representative: String,
    pub balance: String,
    pub link: String,
    pub work: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlocksRequest {
    action: String,
    json_block: bool,
    hashes: Vec<String>,
    include_not_found: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockResponse {
    block_account: String,
    amount: String,
    balance: String,
    pub height: String,
    local_timestamp: String,
    confirmed: String,
    pub contents: Block,
    subtype: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlocksResponse {
    #[serde(flatten)]
    pub data: HashMap<String, BlockResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlocksInfoResponse {
    pub blocks: BlocksResponse,
}

pub fn get_blocks_info(hashes: Vec<String>, node_url: &str) -> BlocksInfoResponse {
    let request = BlocksRequest {
        action: String::from("blocks_info"),
        json_block: true,
        hashes,
        include_not_found: true,
    };
    let body = serde_json::to_string(&request).unwrap();
    //eprintln!("Body: {}", body);
    let response = post_node(body, node_url);

    let blocks_info_response: Result<BlocksInfoResponse, _> = serde_json::from_str(&response);
    match blocks_info_response {
        Ok(blocks_info_response) => blocks_info_response,
        // If deserialisation failed, either there were no blocks
        // Or an different error was encountered.
        Err(_) => BlocksInfoResponse {
            blocks: BlocksResponse {
                data: HashMap::new(),
            },
        },
    }
}
