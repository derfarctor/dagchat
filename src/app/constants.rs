pub const LOCAL_WORK: bool = true;
pub const VERSION: &str = "beta v1.0.0";
pub const SHOW_TO_DP: usize = 6;

pub mod banano {
    pub const DEFAULT_REP: &str =
        "ban_3catgir1p6b1edo5trp7fdb8gsxx4y5ffshbphj73zzy5hu678rsry7srh8b";
    pub const NODE_URL: &str = "https://kaliumapi.appditto.com/api";
    pub const WORK_NODE_URL: &str = "https://kaliumapi.appditto.com/api";
    pub const DIFFICULTY_THRESHOLD: &str = "FFFFFE0000000000";
}

pub mod nano {
    pub const DEFAULT_REP: &str =
        "nano_3zx7rus19yr5qi5zmkawnzo5ehxr7i73xqghhondhfrzftgstgk4gxbubwfq";
    pub const NODE_URL: &str = "https://app.natrium.io/api";
    pub const WORK_NODE_URL: &str = "https://app.natrium.io/api";
    pub const DIFFICULTY_THRESHOLD: &str = "FFFFFFF800000000";
}

pub mod crypto {
    pub const SALT_LENGTH: usize = 16;
    pub const IV_LENGTH: usize = 12;
}

pub mod paths {
    pub const DATA_DIR: &str = "dagchat-beta";
    pub const MESSAGES_DIR: &str = "messages";
    pub const WALLETS: &str = "accounts.dagchat";
    pub const ADDRESS_BOOK: &str = "addressbook.dagchat";
}

pub mod colours {
    use cursive::theme::{BaseColor, Color};
    pub const L_BLUE: Color = Color::Rgb(62, 138, 227);
    pub const M_BLUE: Color = Color::Rgb(0, 106, 255);
    pub const D_BLUE: Color = Color::Rgb(12, 37, 125);

    pub const YELLOW: Color = Color::Light(BaseColor::Yellow);
    pub const OFF_WHITE: Color = Color::Rgb(245, 245, 247);
    pub const RED: Color = Color::Light(BaseColor::Red);
}
