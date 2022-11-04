use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    pub node_url: String,
    pub appditto_node_url: String,
    pub work_server_url: String,
    pub default_rep: String,
    pub send_thresh: String,
    pub receive_thresh: String,
    pub work_type: usize,
    pub save_messages: bool,
}

impl Network {
    pub fn nano() -> Network {
        Network {
            node_url: String::from("https://rainstorm.city/api"),
            appditto_node_url: String::from("https://app.natrium.io/api"),
            work_server_url: String::from("0.0.0.0:7077"),
            default_rep: String::from(
                "nano_3zx7rus19yr5qi5zmkawnzo5ehxr7i73xqghhondhfrzftgstgk4gxbubwfq",
            ),
            send_thresh: String::from("FFFFFFF800000000"),
            receive_thresh: String::from("FFFFFE0000000000"),
            work_type: WorkType::BOOMPOW,
            save_messages: true,
        }
    }
    pub fn banano() -> Network {
        Network {
            node_url: String::from("https://vault.banano.cc/api/node-api"),
            appditto_node_url: String::from("https://kaliumapi.appditto.com/api"),
            work_server_url: String::from("0.0.0.0:7077"),
            default_rep: String::from(
                "ban_3catgir1p6b1edo5trp7fdb8gsxx4y5ffshbphj73zzy5hu678rsry7srh8b",
            ),
            send_thresh: String::from("FFFFFE0000000000"),
            receive_thresh: String::from("FFFFFE0000000000"),
            work_type: WorkType::LOCAL,
            save_messages: true,
        }
    }
}

pub struct WorkType;
impl WorkType {
    pub const BOOMPOW: usize = 0;
    pub const LOCAL: usize = 1;
    pub const WORK_SERVER: usize = 1;
}