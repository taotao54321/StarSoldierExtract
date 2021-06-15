use crate::rom::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SpawnTableEntry {
    Mark(u8),
    Jump(u8),
    Spawn {
        object_id: u8,
        combi: bool, // 前の敵と複合する
        boss: bool,
    },
}

pub fn load_spawn_table(rom: &Rom) -> Vec<SpawnTableEntry> {
    use std::convert::TryFrom;

    const CMD_MARK: u8 = 0xFF;
    const CMD_JUMP: u8 = 0x00;
    const FLAG_BOSS: u8 = 1 << 6;
    const FLAG_NOT_COMBI: u8 = 1 << 7;

    let buf = &rom.prg[prg_offset(0xD30D)..][..0x100];

    let mut res = vec![];
    let mut offset = 0;
    while offset < 0x100 {
        let b = buf[offset];
        offset += 1;

        match b {
            CMD_MARK => res.push(SpawnTableEntry::Mark(u8::try_from(offset).unwrap())),
            CMD_JUMP => {
                let dst = buf[offset];
                offset += 1;
                res.push(SpawnTableEntry::Jump(dst));
            }
            _ => {
                let object_id = b & 0x3F;
                let combi = (b & FLAG_NOT_COMBI) == 0;
                let boss = (b & FLAG_BOSS) != 0;
                res.push(SpawnTableEntry::Spawn {
                    object_id,
                    combi,
                    boss,
                });
            }
        }
    }

    res
}
