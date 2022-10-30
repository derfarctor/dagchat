use super::constants::colours::{L_BLUE, YELLOW};
use crate::app::components::settings::structs::Network;
use cursive::theme::Color;

#[derive(Debug, Clone)]
pub struct Coin {
    pub prefix: String,
    pub name: String,
    pub ticker: String,
    pub multiplier: String,
    pub network: Network,
    pub colour: Color,
}

impl Coin {
    pub fn nano() -> Coin {
        Coin {
            prefix: String::from("nano_"),
            name: String::from("nano"),
            ticker: String::from("Ӿ"),
            multiplier: String::from("1000000000000000000000000000000"),
            network: Network::nano(),
            colour: L_BLUE,
        }
    }
    pub fn banano() -> Coin {
        Coin {
            prefix: String::from("ban_"),
            name: String::from("banano"),
            ticker: String::from(" BAN"),
            multiplier: String::from("100000000000000000000000000000"),
            network: Network::banano(),
            colour: YELLOW,
        }
    }
}

pub struct Coins;

impl Coins {
    pub const NANO: usize = 0;
    pub const BANANO: usize = 1;
}
