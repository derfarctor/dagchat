//
use super::super::super::dcutil::Message;
//
use serde::{Deserialize, Serialize};
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
