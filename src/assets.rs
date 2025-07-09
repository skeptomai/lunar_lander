use macroquad_text::Fonts;
use macroquad::texture::Texture2D;

const GLASS_TTY_VT220: &[u8] = include_bytes!("../assets/fonts/Glass_TTY_VT220.ttf");
const LANDER_UPRIGHT: &[u8] = include_bytes!("../assets/images/lander-upright.png");
const THRUST: &[u8] = include_bytes!("../assets/images/thrust.png");

pub fn load_fonts<'a>() -> Fonts<'a> {
    let mut fonts = Fonts::default();
    fonts
        .load_font_from_bytes("Glass VT200", GLASS_TTY_VT220)
        .unwrap();
    fonts
}

/// Loads lander and thrust texture assets from embedded byte arrays.
///
/// This function loads the two texture components from data embedded directly
/// in the executable at compile time:
/// - Lander texture (main spacecraft body)
/// - Thrust texture (engine flames, rendered beneath lander when thrusting)
///
/// # Returns
///
/// A tuple containing `(lander_texture, thrust_texture)`
///
/// # Panics
///
/// Panics if the embedded texture data cannot be parsed as valid image data
pub fn load_lander_textures() -> (Texture2D, Texture2D) {
    let lander_texture = Texture2D::from_file_with_format(LANDER_UPRIGHT, None);
    let thrust_texture = Texture2D::from_file_with_format(THRUST, None);

    (lander_texture, thrust_texture)
}
