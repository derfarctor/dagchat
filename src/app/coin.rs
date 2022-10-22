use super::constants::colours::*;
use cursive::theme::Color;

#[derive(Debug, Clone)]
pub struct Coin {
    pub prefix: String,
    pub name: String,
    pub ticker: String,
    pub multiplier: String,
    pub node_url: String,
    pub colour: Color,
}

impl Coin {
    pub fn nano() -> Coin {
        Coin {
            prefix: String::from("nano_"),
            name: String::from("nano"),
            ticker: String::from("Ӿ"),
            multiplier: String::from("1000000000000000000000000000000"),
            node_url: String::from("https://app.natrium.io/api"),
            colour: L_BLUE,
        }
    }
    pub fn banano() -> Coin {
        Coin {
            prefix: String::from("ban_"),
            name: String::from("banano"),
            ticker: String::from(" BAN"),
            multiplier: String::from("100000000000000000000000000000"),
            node_url: String::from("https://kaliumapi.appditto.com/api"),
            colour: YELLOW,
        }
    }
}
