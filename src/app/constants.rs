pub const VERSION: &str = "beta v1.0.0";
pub const SHOW_TO_DP: usize = 16;
pub const AUTHOR: &str = "derfarctor (Author)";
pub mod crypto {
    pub const SALT_LENGTH: usize = 16;
    pub const IV_LENGTH: usize = 12;
}

pub mod paths {
    pub const DATA_DIR: &str = "dagchat-beta";
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
