use super::constants::colours::{L_BLUE, YELLOW};
use super::constants::{banano, nano};

use cursive::theme::Color;

#[derive(Debug, Clone)]
pub struct Coin {
    pub prefix: String,
    pub name: String,
    pub ticker: String,
    pub multiplier: String,
    pub node_url: String,
    work_node_url: String,
    pub colour: Color,
}

impl Coin {
    pub fn nano() -> Coin {
        Coin {
            prefix: String::from("nano_"),
            name: String::from("nano"),
            ticker: String::from("Ӿ"),
            multiplier: String::from("1000000000000000000000000000000"),
            node_url: String::from(nano::NODE_URL),
            work_node_url: String::from(nano::WORK_NODE_URL),
            colour: L_BLUE,
        }
    }
    pub fn banano() -> Coin {
        Coin {
            prefix: String::from("ban_"),
            name: String::from("banano"),
            ticker: String::from(" BAN"),
            multiplier: String::from("100000000000000000000000000000"),
            node_url: String::from(banano::NODE_URL),
            work_node_url: String::from(banano::WORK_NODE_URL),
            colour: YELLOW,
        }
    }
}
