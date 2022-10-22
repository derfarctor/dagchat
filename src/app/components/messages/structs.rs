use crate::rpc::blockinfo::BlockResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub head: BlockResponse,
    pub root_hash: String,
    pub blocks: u64,
    pub plaintext: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedMessage {
    // If false, was incoming
    pub outgoing: bool,
    pub address: String,
    pub timestamp: u64,
    pub amount: String,
    pub plaintext: String,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub incoming: bool,
    pub outgoing: bool,
    pub gt_1_raw: bool,
    pub eq_1_raw: bool,
    pub search_term: Option<String>,
}

impl Default for Filter {
    fn default() -> Filter {
        Filter {
            incoming: true,
            outgoing: true,
            gt_1_raw: true,
            eq_1_raw: true,
            search_term: None,
        }
    }
}
