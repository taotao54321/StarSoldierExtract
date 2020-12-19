use std::path::PathBuf;

use image::imageops;
use image::{Rgba, RgbaImage};
use structopt::StructOpt;

use star_soldier_extract::*;

const SPACE_ROW_RANGES: [std::ops::Range<i32>; 3] = [-48..0, -17..0, 1..18];

const COLOR_BG: Rgba<u8> = Rgba([0, 0, 0, 0xFF]);
const COLOR_TEXT: Rgba<u8> = Rgba([0xFF, 0xFF, 0xFF, 0xFF]);

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long)]
    second_round: bool,

    #[structopt(parse(from_os_str))]
    path_rom: PathBuf,

    #[structopt(parse(try_from_str = parse_stage))]
    stage: u8,

    #[structopt(parse(from_os_str))]
    path_out: PathBuf,
}

fn parse_stage(s_stage: &str) -> eyre::Result<u8> {
    const RANGE: std::ops::RangeInclusive<u8> = 1..=16;

    let stage: u8 = s_stage.parse()?;
    eyre::ensure!(RANGE.contains(&stage), "stage must be within {:?}", RANGE);

    Ok(stage)
}

fn range_len(range: &std::ops::Range<i32>) -> i32 {
    range.end - range.start
}

fn canvas() -> RgbaImage {
    let w = 16 * 20 + 32;
    let h = 16 * (256 + itertools::fold(&SPACE_ROW_RANGES, 0, |acc, r| acc + range_len(r))) as u32;

    RgbaImage::from_pixel(w, h, COLOR_BG)
}

fn draw_space(img: &mut RgbaImage, idx: usize) {
    let y_bias = match idx {
        0 => 0,
        1 => 128 + range_len(&SPACE_ROW_RANGES[0]),
        2 => 256 + range_len(&SPACE_ROW_RANGES[0]) + range_len(&SPACE_ROW_RANGES[1]),
        _ => unreachable!(),
    } * 16;

    let font = Font::new(16.0);

    for (i, r) in SPACE_ROW_RANGES[idx].clone().enumerate() {
        let y = img.height() - 16 * (i as u32 + 1) - y_bias as u32;

        font.draw(img, 2, y, COLOR_TEXT, format!("{:3}", r));
    }
}

fn draw_ground(img: &mut RgbaImage, game: &Game, ground: &Ground, idx: usize, second_round: bool) {
    let y_bias = match idx {
        0 => range_len(&SPACE_ROW_RANGES[0]),
        1 => 128 + range_len(&SPACE_ROW_RANGES[0]) + range_len(&SPACE_ROW_RANGES[1]),
        _ => unreachable!(),
    } * 16;

    let plt_set = ground.palette_set_half(idx);
    let imgs_cell = game.cell_images(second_round, plt_set);

    let font = Font::new(16.0);

    for i in 0..128 {
        let r = (i + 128 * idx) as u8;
        let y = img.height() - 16 * (i as u32 + 1) - y_bias as u32;

        for c in 0..20 {
            let visual_id = ground
                .hidden_visual_id(r, c)
                .unwrap_or_else(|| ground.cell(r, c));
            let img_cell = &imgs_cell[visual_id as usize];

            let x = 32 + 16 * c as u32;

            imageops::overlay(img, img_cell, x, y);
        }

        font.draw(img, 2, y, COLOR_TEXT, format!("{:3}", r));
    }
}

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();

    let rom = Rom::from_ines_bytes(std::fs::read(opt.path_rom)?)?;
    let game = Game::from_rom(&rom);

    let ground = game.ground(opt.stage);

    let mut img = canvas();

    for i in 0..3 {
        draw_space(&mut img, i);
    }
    for i in 0..2 {
        draw_ground(&mut img, &game, &ground, i, opt.second_round);
    }

    img.save(opt.path_out)?;

    Ok(())
}
