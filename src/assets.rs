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

/// Loads all lander texture assets asynchronously.
///
/// This function loads the three lander textures:
/// - Normal lander (no thrust)
/// - Acceleration lander (low-medium thrust)
/// - High acceleration lander (high thrust)
///
/// # Returns
///
/// A tuple containing `(normal_texture, accel_texture, high_accel_texture)`
///
/// # Panics
///
/// Panics if any texture file cannot be loaded from the assets directory
pub async fn load_lander_textures() -> (Texture2D, Texture2D, Texture2D) {
    let lander_texture = load_texture("assets/images/lander.png")
        .await
        .expect("Failed to load texture");
    let lander_accel_texture = load_texture("assets/images/lander-accel.png")
        .await
        .expect("Failed to load texture");
    let lander_high_accel_texture = load_texture("assets/images/lander-high-accel.png")
        .await
        .expect("Failed to load texture");

    (lander_texture, lander_accel_texture, lander_high_accel_texture)
}
