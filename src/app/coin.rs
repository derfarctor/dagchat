use super::constants::colours::{L_BLUE, YELLOW};
use cursive::theme::Color;

#[derive(Debug, Clone)]
pub struct Coin {
    pub prefix: String,
    pub name: String,
    pub ticker: String,
    pub multiplier: String,
    pub node_url: String,
    pub work_node_url: String,
    pub default_rep: String,
    pub send_thresh: String,
    pub receive_thresh: String,
    pub colour: Color,
}

impl Coin {
    pub fn nano() -> Coin {
        Coin {
            prefix: String::from("nano_"),
            name: String::from("nano"),
            ticker: String::from("Ӿ"),
            multiplier: String::from("1000000000000000000000000000000"),
            node_url: String::from("https://proxy.nanos.cc/proxy"),
            work_node_url: String::from("https://app.natrium.io/api"),
            default_rep: String::from(
                "nano_3zx7rus19yr5qi5zmkawnzo5ehxr7i73xqghhondhfrzftgstgk4gxbubwfq",
            ),
            send_thresh: String::from("FFFFFFF800000000"),
            receive_thresh: String::from("FFFFFE0000000000"),
            colour: L_BLUE,
        }
    }
    pub fn banano() -> Coin {
        Coin {
            prefix: String::from("ban_"),
            name: String::from("banano"),
            ticker: String::from(" BAN"),
            multiplier: String::from("100000000000000000000000000000"),
            node_url: String::from("https://vault.banano.cc/api/node-api"),
            work_node_url: String::from("https://kaliumapi.appditto.com/api"),
            default_rep: String::from(
                "ban_3catgir1p6b1edo5trp7fdb8gsxx4y5ffshbphj73zzy5hu678rsry7srh8b",
            ),
            send_thresh: String::from("FFFFFE0000000000"),
            receive_thresh: String::from("FFFFFE0000000000"),
            colour: YELLOW,
        }
    }
}
