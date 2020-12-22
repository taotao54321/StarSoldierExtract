use std::convert::TryInto;
use std::io::{self, Read, Write};

use byteorder::{ByteOrder, ReadBytesExt, LE};
use image::imageops;
use image::RgbaImage;
use itertools::iproduct;

use crate::*;

pub const CELL_ZEG_INI: u8 = 0x08;
pub const CELL_TRAP: u8 = 0x96;
pub const CELL_MAX: u8 = 0x96;

pub const META_SPRITE_MAX: u8 = 0x8F;

#[derive(Clone, Debug)]
pub struct Game {
    ground_cells: Vec<Vec<Vec<u8>>>,        // [16][256][20]
    ground_secrets: Vec<Vec<GroundSecret>>, // [16][n]
    ground_configs: Vec<Vec<GroundConfig>>, // [16][2]
    ground_palettes: Vec<Palette>,          // [n]

    cell_visuals: Vec<CellVisual>, // [n]

    sprite_palette_set: Vec<Palette>,           // [4]
    meta_sprite_visuals: Vec<MetaSpriteVisual>, // [n]

    tiles: Vec<Tile>, // [0x800]
}

impl Game {
    pub fn from_rom(rom: &Rom) -> Self {
        Self {
            ground_cells: load_ground_cells(rom),
            ground_secrets: load_ground_secrets(rom),
            ground_configs: load_ground_configs(rom),
            ground_palettes: load_ground_palettes(rom),

            cell_visuals: load_cell_visuals(rom),

            sprite_palette_set: load_sprite_palette_set(rom),
            meta_sprite_visuals: load_meta_sprite_visuals(rom),

            tiles: load_tiles(rom),
        }
    }

    pub fn ground(&self, stage: u8) -> Ground {
        Ground::from_game(self, stage)
    }

    pub fn cell_image(&self, id: u8, second_round: bool, palette_set: &[Palette]) -> RgbaImage {
        let cv = &self.cell_visuals[id as usize];
        let tiles = &self.tiles[(0x100 + if second_round { 0x400 } else { 0 })..];
        cv.to_image(tiles, palette_set)
    }

    pub fn cell_images(&self, second_round: bool, palette_set: &[Palette]) -> Vec<RgbaImage> {
        (0..=CELL_MAX)
            .map(|id| self.cell_image(id, second_round, palette_set))
            .collect()
    }

    pub fn meta_sprite_image(&self, id: u8, second_round: bool) -> RgbaImage {
        let msv = &self.meta_sprite_visuals[id as usize];
        let tiles = &self.tiles[(if second_round { 0x400 } else { 0 })..];
        msv.to_image(tiles, &self.sprite_palette_set)
    }

    pub fn meta_sprite_images(&self, second_round: bool) -> Vec<RgbaImage> {
        (0..=META_SPRITE_MAX)
            .map(|id| self.meta_sprite_image(id, second_round))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Ground {
    cells: Vec<Vec<u8>>,             // [256][20]
    palette_sets: Vec<Vec<Palette>>, // [2][4]
    secrets: Vec<GroundSecret>,      // [n]
}

impl Ground {
    fn from_game(game: &Game, stage: u8) -> Self {
        let idx = stage as usize - 1;

        let configs = &game.ground_configs[idx];

        // ローテート処理
        let mut cells = game.ground_cells[idx].clone();
        for (i, cfg) in itertools::enumerate(configs) {
            if !cfg.is_rotated() {
                continue;
            }

            let r_start = 128 * i;
            let r_end = (128 * (i + 1)).min(240);
            for (r, c) in iproduct!(r_start..r_end, 0..10) {
                cells[r].swap(c, c + 10);
            }
        }

        let palette_sets = configs
            .iter()
            .map(|cfg| {
                cfg.palette_ids()
                    .iter()
                    .map(|&id| game.ground_palettes[id as usize])
                    .collect()
            })
            .collect();

        Self {
            cells,
            palette_sets,
            secrets: game.ground_secrets[idx].clone(),
        }
    }

    /// (r, c) のセルを返す。
    /// そのまま visual_id としても使える。
    pub fn cell(&self, r: u8, c: u8) -> u8 {
        self.cells[r as usize][c as usize]
    }

    /// 隠しセルの visual_id を返す。ゼグも含む。
    pub fn hidden_visual_id(&self, r: u8, c: u8) -> Option<u8> {
        if let Some(secret) = self
            .secrets
            .iter()
            .find(|secret| secret.r == r && secret.c == c)
        {
            return Some(match secret.cell {
                0 => c % 4,
                cell => cell,
            });
        }

        const HIDDEN_ZEG_MIN: u8 = CELL_ZEG_INI + 1;
        const HIDDEN_ZEG_MAX: u8 = CELL_ZEG_INI + 5;
        match self.cell(r, c) {
            HIDDEN_ZEG_MIN..=HIDDEN_ZEG_MAX => Some(CELL_ZEG_INI),
            _ => None,
        }
    }

    pub fn palette_set_half(&self, i: usize) -> &[Palette] {
        &self.palette_sets[i]
    }

    pub fn secrets(&self) -> &[GroundSecret] {
        &self.secrets
    }
}

#[derive(Clone, Debug)]
pub struct GroundSecret {
    r: u8,
    c: u8,
    cell: u8,
}

impl GroundSecret {
    pub fn new(r: u8, c: u8, cell: u8) -> Self {
        Self { r, c, cell }
    }

    pub fn r(&self) -> u8 {
        self.r
    }

    pub fn c(&self) -> u8 {
        self.c
    }

    pub fn cell(&self) -> u8 {
        self.cell
    }
}

/// パレットアニメーションはとりあえず無視する。
#[derive(Clone, Debug)]
pub struct GroundConfig {
    palette_ids: [u8; 4],
    is_rotated: bool,
}

impl GroundConfig {
    pub fn new(palette_ids: [u8; 4], is_rotated: bool) -> Self {
        Self {
            palette_ids,
            is_rotated,
        }
    }

    pub fn from_bytes(buf: impl AsRef<[u8]>) -> Self {
        let buf = &buf.as_ref()[..4];

        let is_rotated = (buf[0] & 0x80) != 0;

        let mut palette_ids = [0; 4];
        for (id, byte) in itertools::zip(&mut palette_ids, buf) {
            *id = byte & 0x3F;
        }

        Self::new(palette_ids, is_rotated)
    }

    pub fn palette_ids(&self) -> &[u8; 4] {
        &self.palette_ids
    }

    pub fn is_rotated(&self) -> bool {
        self.is_rotated
    }
}

/// (左上, 右上, 左下, 右下) の順。
#[derive(Clone, Debug)]
pub struct CellVisual {
    tile_ids: [u8; 4],
    plt_idx: u8,
}

impl CellVisual {
    pub fn new(tile_ids: [u8; 4], plt_idx: u8) -> Self {
        Self { tile_ids, plt_idx }
    }

    pub fn to_image(&self, tiles: &[Tile], palette_set: &[Palette]) -> RgbaImage {
        let plt = palette_set[self.plt_idx as usize];

        let mut img = RgbaImage::new(16, 16);

        for (i, &tile_id) in itertools::enumerate(&self.tile_ids) {
            let img_tile = tiles[tile_id as usize].to_image(plt);
            let x = if i % 2 == 0 { 0 } else { 8 };
            let y = if i / 2 == 0 { 0 } else { 8 };
            imageops::overlay(&mut img, &img_tile, x, y);
        }

        img
    }
}

/// (左上, 左下, 右上, 右下) の順。
#[derive(Clone, Debug)]
pub struct MetaSpriteVisual {
    tile_ids: [u8; 4],
    attrs: [SpriteAttribute; 4],
}

impl MetaSpriteVisual {
    pub fn new(tile_ids: [u8; 4], attrs: [SpriteAttribute; 4]) -> Self {
        Self { tile_ids, attrs }
    }

    pub fn to_image(&self, tiles: &[Tile], palette_set: &[Palette]) -> RgbaImage {
        let mut img = RgbaImage::new(16, 16);

        for i in 0..4 {
            let tile = &tiles[self.tile_ids[i] as usize];
            let attr = self.attrs[i];

            let img_part = sprite_image(tile, attr, palette_set);
            let x = if i / 2 == 0 { 0 } else { 8 };
            let y = if i % 2 == 0 { 0 } else { 8 };
            imageops::overlay(&mut img, &img_part, x, y);
        }

        img
    }
}

fn load_ground_cells(rom: &Rom) -> Vec<Vec<Vec<u8>>> {
    let ptrs = rom.prg[prg_offset(0xD5D9)..]
        .chunks(2 * 2)
        .take(16)
        .map(|buf| [LE::read_u16(&buf[..2]), LE::read_u16(&buf[2..4])]);

    ptrs.map(|ps| {
        let mut cells = Vec::with_capacity(256);
        for &p in &ps {
            load_ground_cells_one_half(&mut cells, rom, p);
        }
        cells
    })
    .collect()
}

fn load_ground_cells_one_half(cells: &mut Vec<Vec<u8>>, rom: &Rom, addr: u16) {
    let mut offset = prg_offset(addr);
    for _ in 0..128 {
        let mut row = Vec::with_capacity(20);
        let wtr = io::Cursor::new(&mut row);

        if rom.prg[offset] == 0xDB {
            let ptr = LE::read_u16(&rom.prg[offset + 1..]);
            let rdr = &rom.prg[prg_offset(ptr)..];
            load_ground_cells_row(rdr, wtr).unwrap();
            offset += 3;
        } else {
            let rdr = &rom.prg[offset..];
            let n_read = load_ground_cells_row(rdr, wtr).unwrap();
            offset += n_read;
        }

        cells.push(row);
    }
}

fn load_ground_cells_row<R: Read, W: Write>(mut rdr: R, mut wtr: W) -> eyre::Result<usize> {
    let mut n_read = 0;
    let mut n_written = 0;

    let mut buf = [0u8; 4];

    macro_rules! read_n {
        ($n:expr) => {
            rdr.read_exact(&mut buf[..$n])?;
            n_read += $n;
        };
    }

    macro_rules! write_n {
        ($n:expr) => {
            wtr.write_all(&buf[..$n])?;
            n_written += $n;
        };
    }

    loop {
        if n_written == 20 {
            return Ok(n_read);
        }

        read_n!(1);
        if buf[0] < 0xDC {
            write_n!(1);
            continue;
        }

        let (unit, count) = match buf[0] {
            0xEE..=0xFF => (1, buf[0] - 0xEB),
            0xE5..=0xED => (2, buf[0] - 0xE3),
            0xE0..=0xE4 => (3, buf[0] - 0xDE),
            0xDC..=0xDF => (4, buf[0] - 0xDA),
            _ => unreachable!(),
        };

        read_n!(unit);

        for _ in 0..count {
            write_n!(unit);
        }
    }
}

fn load_ground_secrets(rom: &Rom) -> Vec<Vec<GroundSecret>> {
    let ptrs = rom.prg[prg_offset(0xD5B9)..]
        .chunks(2)
        .take(16)
        .map(LE::read_u16);

    ptrs.map(|ptr| load_ground_secrets_one(&rom.prg[prg_offset(ptr)..]).unwrap())
        .collect()
}

fn load_ground_secrets_one<R: Read>(mut rdr: R) -> eyre::Result<Vec<GroundSecret>> {
    let mut secrets = Vec::new();

    loop {
        let r = rdr.read_u8()?;
        if r == 0 {
            break;
        }

        let byte = rdr.read_u8()?;
        let c = byte & 0x1F;
        let cell = byte >> 5;

        secrets.push(GroundSecret::new(r, c, cell));
    }

    Ok(secrets)
}

fn load_ground_configs(rom: &Rom) -> Vec<Vec<GroundConfig>> {
    rom.prg[prg_offset(0xD48D)..]
        .chunks(4 * 2)
        .take(16)
        .map(|buf| {
            vec![
                GroundConfig::from_bytes(&buf[..4]),
                GroundConfig::from_bytes(&buf[4..8]),
            ]
        })
        .collect()
}

fn load_ground_palettes(rom: &Rom) -> Vec<Palette> {
    rom.prg[prg_offset(0xD50D)..]
        .chunks(4)
        .take(43)
        .map(Palette::from_bytes)
        .collect()
}

fn load_cell_visuals(rom: &Rom) -> Vec<CellVisual> {
    (0..=CELL_MAX as u16)
        .map(|i| {
            let tile_ids = rom.prg[prg_offset(0xD6B0 + 4 * i)..][..4]
                .try_into()
                .unwrap();
            let plt_idx = rom.prg[prg_offset(0xD619 + i)];
            CellVisual::new(tile_ids, plt_idx)
        })
        .collect()
}

fn load_sprite_palette_set(rom: &Rom) -> Vec<Palette> {
    rom.prg[prg_offset(0xB143)..]
        .chunks(4)
        .take(4)
        .map(Palette::from_bytes)
        .collect()
}

fn load_meta_sprite_visuals(rom: &Rom) -> Vec<MetaSpriteVisual> {
    (0..=META_SPRITE_MAX as u16)
        .map(|i| {
            let buf = &rom.prg[prg_offset(0xC344 + 8 * i)..][..8];
            let mut tile_ids = [0; 4];
            let mut attrs = [SpriteAttribute::from_byte(0); 4];
            for j in 0..4 {
                tile_ids[j] = buf[2 * j];
                attrs[j] = SpriteAttribute::from_byte(buf[2 * j + 1]);
            }
            MetaSpriteVisual::new(tile_ids, attrs)
        })
        .collect()
}

fn load_tiles(rom: &Rom) -> Vec<Tile> {
    rom.chr.chunks(16).map(Tile::from_bytes).collect()
}
