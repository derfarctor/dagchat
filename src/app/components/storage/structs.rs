use serde::{Deserialize, Serialize};
pub struct StorageElements;

impl StorageElements {
    pub const WALLETS: usize = 0;
    pub const LOOKUP: usize = 1;
    pub const ADDRESSBOOK: usize = 2;
}
#[derive(Serialize, Deserialize, Debug)]
pub struct StorageData {
    pub storage_bytes: Vec<Vec<u8>>,
}
