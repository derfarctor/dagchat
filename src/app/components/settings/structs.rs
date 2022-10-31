use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    pub node_url: String,
    pub work_node_url: String,
    pub default_rep: String,
    pub send_thresh: String,
    pub receive_thresh: String,
    pub local_work: bool,
}

impl Network {
    pub fn nano() -> Network {
        Network {
            node_url: String::from("https://proxy.nanos.cc/proxy"),
            work_node_url: String::from("https://app.natrium.io/api"),
            default_rep: String::from(
                "nano_3zx7rus19yr5qi5zmkawnzo5ehxr7i73xqghhondhfrzftgstgk4gxbubwfq",
            ),
            send_thresh: String::from("FFFFFFF800000000"),
            receive_thresh: String::from("FFFFFE0000000000"),
            local_work: false,
        }
    }
    pub fn banano() -> Network {
        Network {
            node_url: String::from("https://vault.banano.cc/api/node-api"),
            work_node_url: String::from("https://kaliumapi.appditto.com/api"),
            default_rep: String::from(
                "ban_3catgir1p6b1edo5trp7fdb8gsxx4y5ffshbphj73zzy5hu678rsry7srh8b",
            ),
            send_thresh: String::from("FFFFFE0000000000"),
            receive_thresh: String::from("FFFFFE0000000000"),
            local_work: false,
        }
    }
}
