use image::{Rgba, RgbaImage};
use once_cell::sync::Lazy;

static FONT: Lazy<rusttype::Font> = Lazy::new(|| {
    rusttype::Font::try_from_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/asset/mplus-1m-regular.ttf"
    )))
    .expect("cannot load font")
});

#[derive(Debug)]
pub struct Font {
    scale: rusttype::Scale,
}

impl Font {
    pub fn new(px: f32) -> Self {
        let scale = rusttype::Scale { x: px, y: px };

        Self { scale }
    }

    pub fn draw(
        &self,
        img: &mut RgbaImage,
        x: u32,
        y: u32,
        color: Rgba<u8>,
        text: impl AsRef<str>,
    ) {
        imageproc::drawing::draw_text_mut(img, color, x, y, self.scale, &FONT, text.as_ref());
    }
}
