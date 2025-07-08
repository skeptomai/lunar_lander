use macroquad_text::Fonts;
use macroquad::texture::load_texture;
use macroquad::texture::Texture2D;

const GLASS_TTY_VT220: &[u8] = include_bytes!("../assets/fonts/Glass_TTY_VT220.ttf");

pub fn load_fonts<'a>() -> Fonts<'a> {
    let mut fonts = Fonts::default();
    fonts
        .load_font_from_bytes("Glass VT200", GLASS_TTY_VT220)
        .unwrap();
    fonts
}

/// Loads lander and thrust texture assets asynchronously.
///
/// This function loads the two texture components:
/// - Lander texture (main spacecraft body)
/// - Thrust texture (engine flames, rendered beneath lander when thrusting)
///
/// # Returns
///
/// A tuple containing `(lander_texture, thrust_texture)`
///
/// # Panics
///
/// Panics if any texture file cannot be loaded from the assets directory
pub async fn load_lander_textures() -> (Texture2D, Texture2D) {
    let lander_texture = load_texture("assets/images/lander-upright.png")
        .await
        .expect("Failed to load lander texture");
    let thrust_texture = load_texture("assets/images/thrust.png")
        .await
        .expect("Failed to load thrust texture");

    (lander_texture, thrust_texture)
}
