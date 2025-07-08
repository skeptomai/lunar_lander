use macroquad_text::Fonts;

const GLASS_TTY_VT220: &[u8] = include_bytes!("../assets/fonts/Glass_TTY_VT220.ttf");

pub fn load_fonts<'a>() -> Fonts<'a> {
    let mut fonts = Fonts::default();
    fonts
        .load_font_from_bytes("Glass VT200", GLASS_TTY_VT220)
        .unwrap();
    fonts
}