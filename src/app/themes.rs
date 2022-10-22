use super::constants::colours::*;
use super::userdata::UserData;

use cursive::theme::{BaseColor, BorderStyle, Color, PaletteColor, Theme};
use cursive::Cursive;

pub fn set_theme(s: &mut Cursive, style: &str, vibrant: bool) {
    let mut theme = s.current_theme().clone();
    if style == "nano" {
        theme = get_nano_theme(theme, vibrant);
    } else {
        theme = get_banano_theme(theme, vibrant);
    }
    s.set_theme(theme);
}

fn get_banano_theme(mut base: Theme, v: bool) -> Theme {
    if v {
        base.shadow = true;
        base.palette[PaletteColor::Background] = YELLOW;
    } else {
        base.palette[PaletteColor::Background] = Color::Rgb(25, 25, 27);
    }
    base.palette[PaletteColor::View] = Color::Rgb(34, 34, 42);
    base.palette[PaletteColor::Primary] = YELLOW;
    base.palette[PaletteColor::Secondary] = YELLOW;
    base.palette[PaletteColor::Tertiary] = OFF_WHITE;
    base.palette[PaletteColor::TitlePrimary] = OFF_WHITE;
    base.palette[PaletteColor::TitleSecondary] = YELLOW;
    base.palette[PaletteColor::Highlight] = Color::Dark(BaseColor::Yellow);
    base.palette[PaletteColor::HighlightInactive] = YELLOW;
    base.palette[PaletteColor::Shadow] = Color::Dark(BaseColor::Yellow);
    base
}

fn get_nano_theme(mut base: Theme, v: bool) -> Theme {
    if v {
        base.shadow = true;
        base.palette[PaletteColor::Background] = L_BLUE;
        base.palette[PaletteColor::Shadow] = D_BLUE;
    } else {
        base.shadow = false;
        base.palette[PaletteColor::Background] = Color::Rgb(25, 25, 27);
    }
    base.borders = BorderStyle::Simple;
    base.palette[PaletteColor::View] = Color::Rgb(34, 34, 42);
    base.palette[PaletteColor::Primary] = OFF_WHITE;
    base.palette[PaletteColor::Secondary] = OFF_WHITE;
    base.palette[PaletteColor::Tertiary] = M_BLUE;
    base.palette[PaletteColor::TitlePrimary] = OFF_WHITE;
    base.palette[PaletteColor::TitleSecondary] = YELLOW;
    base.palette[PaletteColor::Highlight] = D_BLUE;
    base.palette[PaletteColor::HighlightInactive] = L_BLUE;
    base
}

pub fn get_subtitle_colour(s: &mut Cursive) -> Color {
    let data = &s.user_data::<UserData>().unwrap();
    let sub_title_colour;
    if data.coin.colour == YELLOW {
        sub_title_colour = OFF_WHITE;
    } else {
        sub_title_colour = data.coin.colour;
    }
    sub_title_colour
}
