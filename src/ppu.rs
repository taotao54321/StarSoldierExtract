use std::convert::TryInto;

use image::imageops;
use image::{Rgba, RgbaImage};
use once_cell::sync::Lazy;

fn nes_color(id: u8) -> Rgba<u8> {
    static COLORS: Lazy<[Rgba<u8>; 0x40]> = Lazy::new(|| {
        let buf = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/asset/fceux.pal"));
        assert!(buf.len() >= 3 * 0x40, "incomplete NES palette");

        let mut res = [Rgba([0, 0, 0, 0]); 0x40];
        for (e, rgb) in itertools::zip(&mut res, buf.chunks(3).take(0x40)) {
            *e = Rgba([rgb[0], rgb[1], rgb[2], 0xFF]);
        }

        res
    });

    COLORS[id as usize]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Palette([u8; 4]);

impl Palette {
    pub fn new(color_ids: [u8; 4]) -> Self {
        assert!(
            itertools::all(&color_ids, |&id| id < 0x40),
            "invalid color id"
        );

        Self(color_ids)
    }

    pub fn from_bytes(buf: impl AsRef<[u8]>) -> Self {
        Self::new(buf.as_ref()[..4].try_into().expect("incomplete palette"))
    }
}

impl std::ops::Index<usize> for Palette {
    type Output = u8;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tile([u8; 16]);

impl Tile {
    pub fn new(pattern: [u8; 16]) -> Self {
        Self(pattern)
    }

    pub fn from_bytes(buf: impl AsRef<[u8]>) -> Self {
        Self::new(buf.as_ref()[..16].try_into().expect("incomplete pattern"))
    }

    pub fn to_image(&self, plt: Palette, transparent: bool) -> RgbaImage {
        let mut img = RgbaImage::new(8, 8);

        for y in 0..8u32 {
            let byte_lo = self.0[y as usize];
            let byte_hi = self.0[y as usize + 8];
            for x in 0..8u32 {
                let lo = (byte_lo >> (7 - x)) & 1;
                let hi = (byte_hi >> (7 - x)) & 1;
                let idx = lo | (hi << 1);
                if transparent && idx == 0 {
                    continue;
                }
                img.put_pixel(x, y, nes_color(plt[idx as usize]));
            }
        }

        img
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpriteAttribute(u8);

impl SpriteAttribute {
    pub fn from_byte(byte: u8) -> Self {
        Self(byte)
    }

    pub fn palette_index(self) -> u8 {
        self.0 & 3
    }

    pub fn is_behind(self) -> bool {
        (self.0 & (1 << 5)) != 0
    }

    pub fn is_flipped_horizontal(self) -> bool {
        (self.0 & (1 << 6)) != 0
    }

    pub fn is_flipped_vertical(self) -> bool {
        (self.0 & (1 << 7)) != 0
    }
}

pub fn sprite_image(tile: &Tile, attr: SpriteAttribute, palette_set: &[Palette]) -> RgbaImage {
    let plt = palette_set[attr.palette_index() as usize];

    let mut img = tile.to_image(plt, true);

    if attr.is_flipped_horizontal() {
        imageops::flip_horizontal_in_place(&mut img);
    }
    if attr.is_flipped_vertical() {
        imageops::flip_vertical_in_place(&mut img);
    }

    img
}
