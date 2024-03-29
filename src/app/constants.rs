pub const VERSION: &str = "v1.3.0";
pub const SHOW_TO_DP: usize = 12;
pub const AUTHOR: &str = "derfarctor (Author)";
pub const AUTHOR_ADDR: &str = "_3kpznqbuzs3grswcqkzitd5fwky4s5cmyt76wru7kbenfwza7q9c1f1egzhm";
pub const EMPTY_MSG: &str = "Nothing to receive...";
pub const BANANO_MESSAGE_PREAMBLE: &[u8; 10] = &[98, 97, 110, 97, 110, 111, 109, 115, 103, 45]; //utf-8 "bananomsg-" to hex
pub const NANO_MESSAGE_PREAMBLE: &[u8; 8] = &[110, 97, 110, 111, 109, 115, 103, 45]; //utf-8 "nanomsg-" to hex

// In seconds. Used as default timeout.
pub const REQ_TIMEOUT: u64 = 10;

pub mod crypto {
    pub const SALT_LENGTH: usize = 16;
    pub const IV_LENGTH: usize = 12;
}

pub mod paths {
    pub const DATA_DIR: &str = "dagchat";
    pub const MESSAGES_DIR: &str = "messages";
    pub const STORAGE: &str = "storage.dagchat";
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
