use std::path::PathBuf;

use image::imageops;
use image::{Rgba, RgbaImage};
use structopt::StructOpt;

use star_soldier_extract::*;

const COLOR_BG: Rgba<u8> = Rgba([0, 0, 0, 0xFF]);
const COLOR_TEXT: Rgba<u8> = Rgba([0xFF, 0xFF, 0xFF, 0xFF]);

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long)]
    second_round: bool,

    #[structopt(parse(from_os_str))]
    path_rom: PathBuf,

    #[structopt(parse(from_os_str))]
    path_out: PathBuf,
}

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();

    let rom = Rom::from_ines_bytes(std::fs::read(opt.path_rom)?)?;
    let game = Game::from_rom(&rom);

    let mut img = RgbaImage::from_pixel(256 + 16, 144 + 16, COLOR_BG);

    let font = Font::new(16.0);

    for c in 0..16 {
        let x = 16 + 16 * c;
        font.draw(&mut img, x + 2, 0, COLOR_TEXT, format!("x{:X}", c));
    }
    for r in 0..16 {
        let y = 16 + 16 * r;
        font.draw(&mut img, 2, y, COLOR_TEXT, format!("{:X}x", r));
    }

    let imgs_meta_sprite = game.meta_sprite_images(opt.second_round);
    for i in 0..=META_SPRITE_MAX {
        let c = i as u32 % 16;
        let r = i as u32 / 16;
        imageops::overlay(
            &mut img,
            &imgs_meta_sprite[i as usize],
            16 + 16 * c,
            16 + 16 * r,
        );
    }

    img.save(opt.path_out)?;

    Ok(())
}
