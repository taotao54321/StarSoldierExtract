use std::path::PathBuf;

use byteorder::{ByteOrder, LE};
use structopt::StructOpt;

use star_soldier_extract::*;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path_rom: PathBuf,

    #[structopt(parse(try_from_os_str = parse_directory))]
    dir_out: PathBuf,
}

fn parse_directory(s: &std::ffi::OsStr) -> Result<PathBuf, std::ffi::OsString> {
    let dir = PathBuf::from(s);

    dir.is_dir().then(|| dir).ok_or_else(|| s.to_owned())
}

fn load_enemy_param_ptrs(rom: &Rom) -> Vec<u16> {
    rom.prg[prg_offset(0xC804)..]
        .chunks(2)
        .take(0x1F)
        .map(|buf| LE::read_u16(buf))
        .collect()
}

fn enemy_has_bytecode(enemy_id: usize) -> bool {
    assert!((1..=0x1F).contains(&enemy_id));
    enemy_id != 0x11 && enemy_id != 0x14
}

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();

    let rom = Rom::from_ines_bytes(std::fs::read(opt.path_rom)?)?;

    let enemy_param_ptrs = load_enemy_param_ptrs(&rom);
    for (i, &param_ptr) in enemy_param_ptrs.iter().enumerate() {
        let enemy_id = i + 1;
        if !enemy_has_bytecode(enemy_id) {
            continue;
        }

        let bytecode_ptr = LE::read_u16(&rom.prg[prg_offset(param_ptr)..]);
        let bytecode_len = if i == enemy_param_ptrs.len() - 1 {
            0xCA
        } else {
            usize::from(enemy_param_ptrs[i + 1] - bytecode_ptr)
        };
        let bytecode = &rom.prg[prg_offset(bytecode_ptr)..][..bytecode_len];

        let path_out = opt.dir_out.join(format!("bytecode-{:02}.bin", enemy_id));
        std::fs::write(path_out, bytecode)?;
    }

    Ok(())
}
