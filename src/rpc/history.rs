use crate::app::constants::REQ_TIMEOUT;

use super::blockinfo::Block;
use super::process::post_node;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct HistoryRequest {
    action: String,
    account: String,
    count: u64,
    head: String,
    reverse: bool,
    raw: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryResponse {
    account: String,
    history: Vec<Block>,
    #[serde(default)]
    next: String,
}

pub fn get_history(
    target_address: &str,
    head: &str,
    length: u64,
    node_url: &str,
) -> Result<Vec<Block>, String> {
    let request = HistoryRequest {
        action: String::from("account_history"),
        account: String::from(target_address),
        count: length,
        head: String::from(head),
        reverse: true,
        raw: true,
    };
    let body = serde_json::to_string(&request).unwrap();
    let response = post_node(body, node_url, REQ_TIMEOUT)?;
    let history_info: Result<HistoryResponse, _> = serde_json::from_str(&response);
    match history_info {
        Ok(history_info) => Ok(history_info.history),
        Err(e) => {
            //eprintln!("Error getting account history: {}", e);
            Err(e.to_string())
        }
    }
}
