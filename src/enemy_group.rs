use byteorder::{ByteOrder, LE};

use crate::rom::*;

#[derive(Clone, Debug)]
pub struct EnemyGroup {
    pub id: u8,

    pub sprite_idx_base: u8,
    pub difficulty: u8,
    pub shot_with_rank: bool,
    pub accel_shot_with_rank: bool,
    pub homing_shot_with_rank: bool,
    pub extra_act_with_rank: bool,
    pub accel_with_rank: bool,
    pub x_ini: u8,
    pub y_ini: u8,

    pub bytecode: Option<Vec<u8>>,

    pub spawn_interval: u8,
    pub spawn_count: u8,
    pub entrypoints: Vec<u8>,
}

pub fn load_enemy_groups(rom: &Rom) -> Vec<EnemyGroup> {
    let attrs = &rom.prg[prg_offset(0xC7C5)..];
    let difficultys = &rom.prg[prg_offset(0xC7E5)..];
    let param_ptrs = load_enemy_group_param_ptrs(rom);

    (0..=0x1E)
        .map(|i| {
            let attr = attrs[i];
            let shot_with_rank = (attr & (1 << 0)) != 0;
            let accel_shot_with_rank = (attr & (1 << 1)) != 0;
            let homing_shot_with_rank = (attr & (1 << 2)) != 0;
            let extra_act_with_rank = (attr & (1 << 3)) != 0;
            let accel_with_rank = (attr & (1 << 4)) != 0;

            let difficulty = difficultys[i];

            let param = &rom.prg[prg_offset(param_ptrs[i])..];
            let bytecode_ptr = LE::read_u16(&param[0..]);
            let bytecode = (bytecode_ptr >= 0x8000).then(|| {
                let len = if i == 0x1E {
                    0xCA
                } else {
                    usize::from(param_ptrs[i + 1] - bytecode_ptr)
                };
                rom.prg[prg_offset(bytecode_ptr)..][..len].to_vec()
            });
            let x_ini = param[2];
            let y_ini = param[3];
            let sprite_idx_base = param[4];
            let spawn_interval = param[5];
            let spawn_count = param[6];
            let entrypoints = param[7..][..usize::from(spawn_count)].to_vec();

            EnemyGroup {
                id: (i + 1) as u8,

                sprite_idx_base,
                difficulty,
                shot_with_rank,
                accel_shot_with_rank,
                homing_shot_with_rank,
                extra_act_with_rank,
                accel_with_rank,
                x_ini,
                y_ini,

                bytecode,

                spawn_interval,
                spawn_count,
                entrypoints,
            }
        })
        .collect()
}

fn load_enemy_group_param_ptrs(rom: &Rom) -> Vec<u16> {
    rom.prg[prg_offset(0xC804)..]
        .chunks(2)
        .take(0x1F)
        .map(|buf| LE::read_u16(buf))
        .collect()
}
