use super::process::post_node;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInfoResponse {
    pub frontier: String,
    pub open_block: String,
    pub representative_block: String,
    pub balance: String,
    pub modified_timestamp: String,
    pub block_count: String,
    pub account_version: String,
    pub confirmation_height: String,
    pub confirmation_height_frontier: String,
    pub representative: String,
}

pub fn get_account_info(address: &str, node_url: &str) -> Option<AccountInfoResponse> {
    // Change this to AccountInfoRequest struct
    let body_json = json!({
        "action": "account_info",
        "account": String::from(address),
        "representative": true
    });

    let body = body_json.to_string();
    let resp_string = post_node(body, node_url);
    if resp_string.contains("Account not found") {
        return None;
    }
    let account_info: AccountInfoResponse = serde_json::from_str(&resp_string).unwrap();
    Some(account_info)
}

pub fn get_balance(info: &AccountInfoResponse) -> u128 {
    let balance: u128 = info.balance.parse().unwrap();
    balance
}
